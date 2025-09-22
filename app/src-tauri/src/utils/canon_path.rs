use crate::errors::FileError;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonPath {
    path: PathBuf,
}

//FIXME: Turn this into try_from as this may fail
impl From<PathBuf> for CanonPath {
    fn from(path: PathBuf) -> CanonPath {
        let p = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => PathBuf::new(),
        };
        CanonPath { path: p }
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
    pub fn try_exists(&self) -> Result<bool> {
        Ok(self.path.try_exists()?)
    }

    pub fn as_str(&self) -> Result<&str> {
        match self.path.to_str() {
            Some(str) => Ok(str),
            None => Err(FileError::PathNotFoundError {
                path: self.path.to_string_lossy().to_string(),
            })?,
        }
    }
}
