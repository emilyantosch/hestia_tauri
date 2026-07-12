use anyhow::{Context, Result};
use library::library::{Library, LibraryConfig, LibraryPathConfig};
use migration::{Migrator, MigratorTrait};
use repositories::config::DatabaseSettings;
use repositories::fs::operations::FileRepository;
use repositories::manager::DatabaseManager;
use std::fmt::{self, Display, Formatter};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryName(String);

impl LibraryName {
    fn parse(value: &str) -> ControllerResult<Self> {
        let value = value.trim();
        let mut components = Path::new(value).components();
        if value.is_empty()
            || !matches!(components.next(), Some(Component::Normal(_)))
            || components.next().is_some()
        {
            return Err(ControllerError::InvalidLibraryName);
        }
        Ok(Self(value.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for LibraryName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LibraryInfo {
    name: LibraryName,
    path: PathBuf,
}

impl LibraryInfo {
    fn from_path(path: PathBuf) -> ControllerResult<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(ControllerError::InvalidLibraryFolderName)?;
        Ok(Self {
            name: LibraryName(name.to_string()),
            path,
        })
    }

    #[must_use]
    pub fn name(&self) -> &LibraryName {
        &self.name
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ControllerOperation {
    OpenDataDirectory,
    ListLibraries,
    OpenLibrary,
    CreateLibrary,
    InitializeLibrary,
}

impl ControllerOperation {
    fn action(self) -> &'static str {
        match self {
            Self::OpenDataDirectory => "Could not open the application data directory",
            Self::ListLibraries => "Could not list libraries",
            Self::OpenLibrary => "Could not open the library",
            Self::CreateLibrary => "Could not create the library",
            Self::InitializeLibrary => "Could not initialize the library",
        }
    }
}

#[derive(Debug)]
pub enum ControllerError {
    InvalidLibraryName,
    InvalidLibraryFolderName,
    NotHestiaLibrary,
    InvalidContentFolder,
    LibraryAlreadyExists,
    NoLibrarySelected,
    MissingStorageFolder,
    NoContentFolders,
    OperationFailed {
        operation: ControllerOperation,
        source: anyhow::Error,
    },
}

impl ControllerError {
    fn operation(operation: ControllerOperation, source: impl Into<anyhow::Error>) -> Self {
        Self::OperationFailed {
            operation,
            source: source.into(),
        }
    }
}

impl Display for ControllerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLibraryName => {
                formatter.write_str("Library name must be a single folder name.")
            }
            Self::InvalidLibraryFolderName => {
                formatter.write_str("A library has an invalid folder name.")
            }
            Self::NotHestiaLibrary => {
                formatter.write_str("The selected folder is not a Hestia library.")
            }
            Self::InvalidContentFolder => {
                formatter.write_str("The library content path must be a folder.")
            }
            Self::LibraryAlreadyExists => {
                formatter.write_str("A library with that name already exists.")
            }
            Self::NoLibrarySelected => {
                formatter.write_str("Select a library before initializing it.")
            }
            Self::MissingStorageFolder => formatter.write_str("The library has no storage folder."),
            Self::NoContentFolders => formatter.write_str("The library has no content folders."),
            Self::OperationFailed { operation, source } => {
                write!(formatter, "{}: {source:#}", operation.action())
            }
        }
    }
}

impl std::error::Error for ControllerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OperationFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

type ControllerResult<T> = std::result::Result<T, ControllerError>;

#[derive(Debug)]
struct Workspace {
    database_manager: Arc<DatabaseManager>,
    file_operations: Arc<FileRepository>,
}

#[derive(Debug)]
enum AppState {
    AwaitingLibrary,
    Ready {
        library: Library,
        workspace: Workspace,
    },
}

#[derive(Debug)]
pub struct AppController {
    data_home: PathBuf,
    state: Mutex<AppState>,
}

impl AppController {
    pub fn new() -> ControllerResult<Self> {
        let data_home = library::io::create_or_validate_data_directory().map_err(|error| {
            ControllerError::operation(ControllerOperation::OpenDataDirectory, error)
        })?;
        Self::new_in(data_home)
    }

    pub fn new_in(data_home: impl AsRef<Path>) -> ControllerResult<Self> {
        std::fs::create_dir_all(data_home.as_ref()).map_err(|error| {
            ControllerError::operation(ControllerOperation::OpenDataDirectory, error)
        })?;
        let data_home = data_home.as_ref().canonicalize().map_err(|error| {
            ControllerError::operation(ControllerOperation::OpenDataDirectory, error)
        })?;
        std::fs::create_dir_all(data_home.join("hestia")).map_err(|error| {
            ControllerError::operation(ControllerOperation::OpenDataDirectory, error)
        })?;

        Ok(Self {
            data_home,
            state: Mutex::new(AppState::AwaitingLibrary),
        })
    }

    pub fn list_libraries(&self) -> ControllerResult<Vec<LibraryInfo>> {
        let mut libraries = Library::list_libraries_in(&self.data_home)
            .map_err(|error| ControllerError::operation(ControllerOperation::ListLibraries, error))?
            .into_iter()
            .map(PathBuf::from)
            .map(LibraryInfo::from_path)
            .collect::<ControllerResult<Vec<_>>>()?;
        libraries.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
        Ok(libraries)
    }

    pub async fn select_library(&self, path: impl AsRef<Path>) -> ControllerResult<LibraryInfo> {
        let path = path
            .as_ref()
            .canonicalize()
            .map_err(|error| ControllerError::operation(ControllerOperation::OpenLibrary, error))?;
        let libraries_root = self.data_home.join("hestia");
        if !path.starts_with(&libraries_root)
            || !path.is_dir()
            || !path.join("config.toml").is_file()
        {
            return Err(ControllerError::NotHestiaLibrary);
        }

        let library = Library::new_in(&self.data_home)
            .switch_or_create_lib_in(&path, &self.data_home)
            .map_err(|error| ControllerError::operation(ControllerOperation::OpenLibrary, error))?;
        self.activate(library).await
    }

    pub async fn create_library(
        &self,
        name: &str,
        content_path: impl AsRef<Path>,
    ) -> ControllerResult<LibraryInfo> {
        let name = LibraryName::parse(name)?;
        let content_path = content_path
            .as_ref()
            .canonicalize()
            .map_err(|_| ControllerError::InvalidContentFolder)?;
        if !content_path.is_dir() {
            return Err(ControllerError::InvalidContentFolder);
        }

        let share_path = self.data_home.join("hestia").join(name.as_str());
        if share_path.try_exists().map_err(|error| {
            ControllerError::operation(ControllerOperation::CreateLibrary, error)
        })? {
            return Err(ControllerError::LibraryAlreadyExists);
        }

        let content_name = content_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("Folder")
            .to_string();
        let mut config = LibraryConfig::default();
        config.name = name.to_string();
        config.library_paths = vec![LibraryPathConfig {
            name: Some(content_name),
            path: Some(content_path),
        }];

        let mut library = Library::new_in(&self.data_home);
        library.share_path = Some(share_path);
        library.library_config = Some(config);
        library.save_config().map_err(|error| {
            ControllerError::operation(ControllerOperation::CreateLibrary, error)
        })?;

        self.activate(library).await
    }

    pub async fn initialize_workspace(&self) -> ControllerResult<()> {
        let (database_manager, file_operations, library_paths) = {
            let state = self.state.lock().await;
            let AppState::Ready { library, workspace } = &*state else {
                return Err(ControllerError::NoLibrarySelected);
            };
            let library_paths = library
                .library_config
                .as_ref()
                .map(|config| {
                    config
                        .library_paths
                        .iter()
                        .filter_map(|path| path.path.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            (
                Arc::clone(&workspace.database_manager),
                Arc::clone(&workspace.file_operations),
                library_paths,
            )
        };

        Migrator::up(database_manager.get_connection().as_ref(), None)
            .await
            .context("database migration failed")
            .map_err(|error| {
                ControllerError::operation(ControllerOperation::InitializeLibrary, error)
            })?;
        if library_paths.is_empty() {
            return Err(ControllerError::NoContentFolders);
        }
        file_operations
            .upsert_root_folders(library_paths)
            .await
            .map_err(|error| {
                ControllerError::operation(ControllerOperation::InitializeLibrary, error)
            })?;
        Ok(())
    }

    async fn activate(&self, library: Library) -> ControllerResult<LibraryInfo> {
        let info = LibraryInfo::from_path(
            library
                .share_path
                .clone()
                .ok_or(ControllerError::MissingStorageFolder)?,
        )?;
        let workspace = Workspace::open(&library)
            .await
            .map_err(|error| ControllerError::operation(ControllerOperation::OpenLibrary, error))?;
        *self.state.lock().await = AppState::Ready { library, workspace };
        Ok(info)
    }
}

impl Workspace {
    async fn open(library: &Library) -> Result<Self> {
        let database_path = library.get_canon_database_path()?;
        let connection_string = format!("sqlite:///{}", database_path.as_str()?);
        let settings = DatabaseSettings {
            con_string: connection_string,
            timeout: 30_000,
            ..DatabaseSettings::default()
        };
        let database_manager = Arc::new(DatabaseManager::new(settings).await?);
        database_manager.test_connection().await?;
        let file_operations = Arc::new(FileRepository::new(Arc::clone(&database_manager)));
        Ok(Self {
            database_manager,
            file_operations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AppController, ControllerError};
    use anyhow::Result;
    use tempfile::TempDir;

    #[tokio::test]
    async fn create_library_returns_an_owned_summary() -> Result<()> {
        let data_home = TempDir::new()?;
        let content = TempDir::new()?;
        let controller = AppController::new_in(data_home.path())?;

        let library = controller.create_library("Photos", content.path()).await?;

        assert_eq!(library.name().as_str(), "Photos");
        assert_eq!(library.path(), data_home.path().join("hestia/Photos"));
        assert!(library.path().join("config.toml").is_file());
        assert!(library.path().join("db.sqlite").is_file());
        Ok(())
    }

    #[tokio::test]
    async fn list_libraries_returns_only_library_directories() -> Result<()> {
        let data_home = TempDir::new()?;
        let content = TempDir::new()?;
        let controller = AppController::new_in(data_home.path())?;
        controller.create_library("Photos", content.path()).await?;
        std::fs::write(data_home.path().join("hestia/not-a-library"), "ignored")?;

        let libraries = controller.list_libraries()?;

        assert_eq!(libraries.len(), 1);
        assert_eq!(libraries[0].name().as_str(), "Photos");
        Ok(())
    }

    #[tokio::test]
    async fn select_library_opens_an_existing_library() -> Result<()> {
        let data_home = TempDir::new()?;
        let content = TempDir::new()?;
        let creator = AppController::new_in(data_home.path())?;
        let created = creator.create_library("Photos", content.path()).await?;
        let controller = AppController::new_in(data_home.path())?;

        let selected = controller.select_library(created.path()).await?;

        assert_eq!(selected, created);
        Ok(())
    }

    #[tokio::test]
    async fn initialize_workspace_migrates_the_selected_database() -> Result<()> {
        let data_home = TempDir::new()?;
        let content = TempDir::new()?;
        let controller = AppController::new_in(data_home.path())?;
        let library = controller.create_library("Photos", content.path()).await?;

        controller.initialize_workspace().await?;

        assert!(std::fs::metadata(library.path().join("db.sqlite"))?.len() > 0);
        Ok(())
    }

    #[tokio::test]
    async fn initialize_workspace_reports_when_no_library_is_selected() -> Result<()> {
        let data_home = TempDir::new()?;
        let controller = AppController::new_in(data_home.path())?;

        let error = controller
            .initialize_workspace()
            .await
            .expect_err("initialization should require a selected library");

        assert!(matches!(error, ControllerError::NoLibrarySelected));
        Ok(())
    }

    #[tokio::test]
    async fn create_library_rejects_path_traversal_names() -> Result<()> {
        let data_home = TempDir::new()?;
        let content = TempDir::new()?;
        let controller = AppController::new_in(data_home.path())?;

        let error = controller
            .create_library("../Photos", content.path())
            .await
            .expect_err("library names must not escape the library directory");

        assert!(matches!(error, ControllerError::InvalidLibraryName));
        Ok(())
    }
}
