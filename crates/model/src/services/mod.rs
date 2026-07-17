use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

pub mod decorations;
pub mod file;
pub mod folder;
pub mod tag;
pub mod thumbnail;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonPath {
    path: PathBuf,
}

impl TryFrom<PathBuf> for CanonPath {
    type Error = anyhow::Error;
    fn try_from(path: PathBuf) -> Result<CanonPath> {
        Ok(CanonPath {
            path: path.canonicalize()?,
        })
    }
}

impl From<CanonPath> for PathBuf {
    fn from(path: CanonPath) -> PathBuf {
        path.path
    }
}

impl AsRef<Path> for CanonPath {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl CanonPath {
    pub fn try_exists(&self) -> Result<bool> {
        Ok(self.path.try_exists()?)
    }

    pub fn as_str(&self) -> Result<&str> {
        match self.path.to_str() {
            Some(str) => Ok(str),
            None => bail!("CanonPath could not be converted to &str"),
        }
    }
}
