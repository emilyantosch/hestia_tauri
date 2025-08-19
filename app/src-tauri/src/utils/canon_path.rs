use crate::errors::FileError;
use std::path::PathBuf;

pub struct CanonPath {
    path: PathBuf,
}

impl TryFrom<PathBuf> for CanonPath {
    type Error = FileError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Ok(CanonPath {
            path: std::fs::canonicalize(path)?,
        })
    }
}

impl TryInto<PathBuf> for CanonPath {
    type Error = FileError;

    fn try_into(self) -> Result<PathBuf, Self::Error> {
        Ok(self.path)
    }
}
