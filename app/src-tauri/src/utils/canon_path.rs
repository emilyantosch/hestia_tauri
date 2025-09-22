use crate::errors::FileError;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonPath {
    path: PathBuf,
}

impl From<PathBuf> for CanonPath {
    fn from(path: PathBuf) -> CanonPath {
        CanonPath { path }
    }
}

impl From<CanonPath> for PathBuf {
    fn from(path: CanonPath) -> PathBuf {
        path.path
    }
}

impl AsRef<Path> for CanonPath {
    fn as_ref(&self) -> &Path {
        &self.path.as_path()
    }
}

impl CanonPath {
    pub fn try_exists(&self) -> Result<bool, FileError> {
        Ok(self.path.try_exists()?)
    }
}
