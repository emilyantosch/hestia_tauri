use controllers::{AppController, FileInfo, FolderInfo, LibraryInfo, TagInfo};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QStringList, QUrl, QVariant,
};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::runtime::Handle;

#[derive(Clone)]
struct AppContext {
    controller: Arc<AppController>,
    runtime: Handle,
}

static CONTEXT: OnceLock<AppContext> = OnceLock::new();

pub fn initialize(controller: Arc<AppController>, runtime: Handle) -> Result<(), &'static str> {
    CONTEXT
        .set(AppContext {
            controller,
            runtime,
        })
        .map_err(|_| "application context was already initialized")
}

#[cxx_qt::bridge]
mod ffi {
    unsafe extern "C++" {
        include!(<QAbstractListModel>);
        type QAbstractListModel;

        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[qenum(FolderModel)]
    enum FolderRole {
        Id,
        Name,
        Path,
        ParentId,
    }

    #[qenum(FileModel)]
    enum FileRole {
        Id,
        Name,
        Path,
        Type,
        ThumbnailUrl,
    }

    #[qenum(TagModel)]
    enum TagRole {
        Id,
        Name,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QStringList, library_names, cxx_name = "libraryNames")]
        #[qproperty(bool, busy)]
        #[qproperty(bool, ready)]
        #[qproperty(QString, error)]
        #[qproperty(QString, status)]
        type HestiaBackend = super::HestiaBackendRust;

        #[qinvokable]
        #[cxx_name = "refreshLibraries"]
        fn refresh_libraries(self: Pin<&mut HestiaBackend>);
        #[qinvokable]
        #[cxx_name = "openLibrary"]
        fn open_library(self: Pin<&mut HestiaBackend>, index: i32);
        #[qinvokable]
        #[cxx_name = "createLibrary"]
        fn create_library(self: Pin<&mut HestiaBackend>, name: &QString, folder: &QUrl);
        #[qinvokable]
        fn scan(self: Pin<&mut HestiaBackend>);

        #[qsignal]
        #[cxx_name = "operationFinished"]
        fn operation_finished(self: Pin<&mut HestiaBackend>);

        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type FolderModel = super::FolderModelRust;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FolderModel, parent: &QModelIndex) -> i32;
        #[cxx_override]
        fn data(self: &FolderModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FolderModel) -> QHash_i32_QByteArray;
        #[qinvokable]
        fn refresh(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FolderModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FolderModel>);

        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        type FileModel = super::FileModelRust;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &FileModel, parent: &QModelIndex) -> i32;
        #[cxx_override]
        fn data(self: &FileModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &FileModel) -> QHash_i32_QByteArray;
        #[qinvokable]
        fn refresh(self: Pin<&mut FileModel>, folder_id: i32, search: &QString);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut FileModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut FileModel>);

        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(QString, error)]
        type TagModel = super::TagModelRust;

        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &TagModel, parent: &QModelIndex) -> i32;
        #[cxx_override]
        fn data(self: &TagModel, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &TagModel) -> QHash_i32_QByteArray;
        #[qinvokable]
        fn refresh(self: Pin<&mut TagModel>);
        #[qinvokable]
        fn create(self: Pin<&mut TagModel>, name: &QString);
        #[qinvokable]
        fn rename(self: Pin<&mut TagModel>, tag_id: i32, name: &QString);
        #[qinvokable]
        fn remove(self: Pin<&mut TagModel>, tag_id: i32);
        #[qinvokable]
        fn assign(self: Pin<&mut TagModel>, file_id: i32, tag_id: i32);
        #[qinvokable]
        fn unassign(self: Pin<&mut TagModel>, file_id: i32, tag_id: i32);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut TagModel>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut TagModel>);
    }

    impl cxx_qt::Threading for HestiaBackend {}
    impl cxx_qt::Threading for FolderModel {}
    impl cxx_qt::Threading for FileModel {}
    impl cxx_qt::Threading for TagModel {}
}

#[derive(Default)]
pub struct HestiaBackendRust {
    library_names: QStringList,
    busy: bool,
    ready: bool,
    error: QString,
    status: QString,
    libraries: Vec<LibraryInfo>,
}

impl ffi::HestiaBackend {
    fn refresh_libraries(mut self: Pin<&mut Self>) {
        let Some(context) = CONTEXT.get() else {
            self.set_error("Backend is not initialized.".into());
            return;
        };
        match context.controller.list_libraries() {
            Ok(libraries) => {
                let names = libraries
                    .iter()
                    .map(|library| QString::from(library.name().as_str()))
                    .collect();
                self.as_mut().rust_mut().libraries = libraries;
                self.as_mut().set_library_names(names);
                self.set_error(QString::default());
            }
            Err(error) => self.set_error(error.to_string().into()),
        }
    }

    fn open_library(self: Pin<&mut Self>, index: i32) {
        let Some(path) = usize::try_from(index)
            .ok()
            .and_then(|index| self.rust().libraries.get(index))
            .map(|library| library.path().to_path_buf())
        else {
            self.set_error("Select a valid library.".into());
            return;
        };
        self.start_library_task(None, path);
    }

    fn create_library(self: Pin<&mut Self>, name: &QString, folder: &QUrl) {
        let Some(path) = folder.to_local_file() else {
            self.set_error("Choose a local folder.".into());
            return;
        };
        self.start_library_task(Some(name.to_string()), PathBuf::from(path.to_string()));
    }

    fn start_library_task(mut self: Pin<&mut Self>, name: Option<String>, path: PathBuf) {
        let Some(context) = CONTEXT.get().cloned() else {
            self.set_error("Backend is not initialized.".into());
            return;
        };
        if *self.busy() {
            return;
        }
        self.as_mut().set_busy(true);
        self.as_mut().set_error(QString::default());
        self.as_mut().set_status("Opening library…".into());
        let qt_thread = self.qt_thread();
        context.runtime.spawn(async move {
            let result = match name {
                Some(name) => context.controller.create_library(&name, path).await,
                None => context.controller.select_library(path).await,
            };
            let result = match result {
                Ok(_) => context.controller.initialize_workspace().await,
                Err(error) => Err(error),
            };
            let result = match result {
                Ok(()) => context.controller.scan().await.map(|_| ()),
                Err(error) => Err(error),
            };
            if result.is_ok() {
                let _thumbnail_result = context.controller.generate_thumbnails().await;
            }
            if result.is_ok() {
                match context.controller.start_watching() {
                    Ok(mut changes) => {
                        let watcher_thread = qt_thread.clone();
                        let controller = Arc::clone(&context.controller);
                        tokio::spawn(async move {
                            while changes.recv().await.is_some() {
                                let _thumbnail_result = controller.generate_thumbnails().await;
                                drop(watcher_thread.queue(|mut backend| {
                                    backend.as_mut().set_status("Library updated".into());
                                    backend.as_mut().operation_finished();
                                }));
                            }
                        });
                    }
                    Err(error) => {
                        drop(qt_thread.queue(move |mut backend| {
                            backend.as_mut().set_error(error.to_string().into());
                        }));
                    }
                }
            }
            drop(qt_thread.queue(move |mut backend| {
                backend.as_mut().set_busy(false);
                match result {
                    Ok(()) => {
                        backend.as_mut().set_ready(true);
                        backend.as_mut().set_status("Library ready".into());
                        backend.as_mut().set_error(QString::default());
                    }
                    Err(error) => {
                        backend.as_mut().set_ready(false);
                        backend.as_mut().set_status(QString::default());
                        backend.as_mut().set_error(error.to_string().into());
                    }
                }
                backend.as_mut().operation_finished();
            }));
        });
    }

    fn scan(mut self: Pin<&mut Self>) {
        let Some(context) = CONTEXT.get().cloned() else {
            self.set_error("Backend is not initialized.".into());
            return;
        };
        if *self.busy() {
            return;
        }
        self.as_mut().set_busy(true);
        self.as_mut().set_status("Scanning…".into());
        let qt_thread = self.qt_thread();
        context.runtime.spawn(async move {
            let result = context.controller.scan().await;
            if result.is_ok() {
                let _thumbnail_result = context.controller.generate_thumbnails().await;
            }
            drop(qt_thread.queue(move |mut backend| {
                backend.as_mut().set_busy(false);
                match result {
                    Ok(report) => backend.as_mut().set_status(
                        format!(
                            "Scanned {} files; {} changes",
                            report.files_scanned(),
                            report.changed()
                        )
                        .into(),
                    ),
                    Err(error) => backend.as_mut().set_error(error.to_string().into()),
                }
                backend.as_mut().operation_finished();
            }));
        });
    }
}

#[derive(Default)]
pub struct FolderModelRust {
    items: Vec<FolderInfo>,
}

impl ffi::FolderModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(item) = usize::try_from(index.row())
            .ok()
            .and_then(|row| self.items.get(row))
        else {
            return QVariant::default();
        };
        match (ffi::FolderRole { repr: role }) {
            ffi::FolderRole::Id => (&item.id()).into(),
            ffi::FolderRole::Name => (&QString::from(item.name())).into(),
            ffi::FolderRole::Path => {
                (&QString::from(&item.path().to_string_lossy().to_string())).into()
            }
            ffi::FolderRole::ParentId => (&item.parent_id().unwrap_or(-1)).into(),
            _ => QVariant::default(),
        }
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let _ = self;
        roles(&[
            (ffi::FolderRole::Id.repr, "id"),
            (ffi::FolderRole::Name.repr, "name"),
            (ffi::FolderRole::Path.repr, "path"),
            (ffi::FolderRole::ParentId.repr, "parentId"),
        ])
    }

    fn refresh(self: Pin<&mut Self>) {
        let Some(context) = CONTEXT.get().cloned() else {
            return;
        };
        let qt_thread = self.qt_thread();
        context.runtime.spawn(async move {
            let result = context.controller.list_folders().await;
            drop(qt_thread.queue(move |mut model| {
                if let Ok(items) = result {
                    reset_folders(model.as_mut(), items);
                }
            }));
        });
    }
}

#[derive(Default)]
pub struct FileModelRust {
    items: Vec<FileInfo>,
}

impl ffi::FileModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(item) = usize::try_from(index.row())
            .ok()
            .and_then(|row| self.items.get(row))
        else {
            return QVariant::default();
        };
        match (ffi::FileRole { repr: role }) {
            ffi::FileRole::Id => (&item.id()).into(),
            ffi::FileRole::Name => (&QString::from(item.name())).into(),
            ffi::FileRole::Path => {
                (&QString::from(&item.path().to_string_lossy().to_string())).into()
            }
            ffi::FileRole::Type => (&item.file_type_id()).into(),
            ffi::FileRole::ThumbnailUrl => {
                item.thumbnail_path()
                    .map_or_else(QVariant::default, |path| {
                        (&QUrl::from_local_file(&QString::from(
                            &path.to_string_lossy().to_string(),
                        )))
                            .into()
                    })
            }
            _ => QVariant::default(),
        }
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let _ = self;
        roles(&[
            (ffi::FileRole::Id.repr, "id"),
            (ffi::FileRole::Name.repr, "name"),
            (ffi::FileRole::Path.repr, "path"),
            (ffi::FileRole::Type.repr, "type"),
            (ffi::FileRole::ThumbnailUrl.repr, "thumbnailUrl"),
        ])
    }

    fn refresh(self: Pin<&mut Self>, folder_id: i32, search: &QString) {
        let Some(context) = CONTEXT.get().cloned() else {
            return;
        };
        let folder_id = (folder_id >= 0).then_some(folder_id);
        let search = search.to_string();
        let qt_thread = self.qt_thread();
        context.runtime.spawn(async move {
            let result = context.controller.list_files(folder_id, &search).await;
            drop(qt_thread.queue(move |mut model| {
                if let Ok(items) = result {
                    reset_files(model.as_mut(), items);
                }
            }));
        });
    }
}

#[derive(Default)]
pub struct TagModelRust {
    error: QString,
    items: Vec<TagInfo>,
}

impl ffi::TagModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.items.len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let Some(item) = usize::try_from(index.row())
            .ok()
            .and_then(|row| self.items.get(row))
        else {
            return QVariant::default();
        };
        match (ffi::TagRole { repr: role }) {
            ffi::TagRole::Id => (&item.id()).into(),
            ffi::TagRole::Name => (&QString::from(item.name())).into(),
            _ => QVariant::default(),
        }
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let _ = self;
        roles(&[
            (ffi::TagRole::Id.repr, "id"),
            (ffi::TagRole::Name.repr, "name"),
        ])
    }

    fn refresh(self: Pin<&mut Self>) {
        self.run_tag_action(|controller| async move { controller.list_tags().await });
    }

    fn create(self: Pin<&mut Self>, name: &QString) {
        let name = name.to_string();
        self.run_tag_action(move |controller| async move {
            controller.create_tag(&name).await?;
            controller.list_tags().await
        });
    }

    fn rename(self: Pin<&mut Self>, tag_id: i32, name: &QString) {
        let name = name.to_string();
        self.run_tag_action(move |controller| async move {
            controller.update_tag(tag_id, &name).await?;
            controller.list_tags().await
        });
    }

    fn remove(self: Pin<&mut Self>, tag_id: i32) {
        self.run_tag_action(move |controller| async move {
            controller.delete_tag(tag_id).await?;
            controller.list_tags().await
        });
    }

    fn assign(self: Pin<&mut Self>, file_id: i32, tag_id: i32) {
        self.run_tag_action(move |controller| async move {
            controller.assign_tag(file_id, tag_id).await?;
            controller.list_tags().await
        });
    }

    fn unassign(self: Pin<&mut Self>, file_id: i32, tag_id: i32) {
        self.run_tag_action(move |controller| async move {
            controller.remove_tag(file_id, tag_id).await?;
            controller.list_tags().await
        });
    }

    fn run_tag_action<F, Fut>(self: Pin<&mut Self>, action: F)
    where
        F: FnOnce(Arc<AppController>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Vec<TagInfo>, controllers::ControllerError>> + Send + 'static,
    {
        let Some(context) = CONTEXT.get().cloned() else {
            return;
        };
        let qt_thread = self.qt_thread();
        context.runtime.spawn(async move {
            let result = action(context.controller).await;
            drop(qt_thread.queue(move |mut model| match result {
                Ok(items) => {
                    model.as_mut().set_error(QString::default());
                    reset_tags(model.as_mut(), items);
                }
                Err(error) => model.as_mut().set_error(error.to_string().into()),
            }));
        });
    }
}

fn roles(values: &[(i32, &str)]) -> QHash<QHashPair_i32_QByteArray> {
    let mut roles = QHash::default();
    for (role, name) in values {
        roles.insert(*role, QByteArray::from(*name));
    }
    roles
}

// ponytail: reset small models; add row-level diffs only when profiling shows a need.
fn reset_folders(mut model: Pin<&mut ffi::FolderModel>, items: Vec<FolderInfo>) {
    unsafe { model.as_mut().begin_reset_model() };
    model.as_mut().rust_mut().items = items;
    unsafe { model.as_mut().end_reset_model() };
}

fn reset_files(mut model: Pin<&mut ffi::FileModel>, items: Vec<FileInfo>) {
    unsafe { model.as_mut().begin_reset_model() };
    model.as_mut().rust_mut().items = items;
    unsafe { model.as_mut().end_reset_model() };
}

fn reset_tags(mut model: Pin<&mut ffi::TagModel>, items: Vec<TagInfo>) {
    unsafe { model.as_mut().begin_reset_model() };
    model.as_mut().rust_mut().items = items;
    unsafe { model.as_mut().end_reset_model() };
}
