use blake3::{Hash as Blake3Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::errors::AppError;
use crate::errors::{HashError, HashErrorKind};
use crate::file_system::FileId;

pub struct FileHash {
    pub content_hash: Blake3Hash,
    pub identity_hash: Blake3Hash,
    pub file_id: FileId,
}

pub struct FolderHash {
    pub structure_hash: Blake3Hash,
    pub content_hash: Blake3Hash,
    pub identity_hash: Blake3Hash,
    pub file_id: FileId,
}

impl FileHash {
    pub async fn compute(path: &Path) -> Result<FileHash, AppError> {
        let file_id = FileId::extract(path).await?;

        let content_hash = Self::hash_file_content(path).await?;

        let identity_hash = Self::hash_identity(
            &content_hash,
            &file_id,
            path.file_name().and_then(|n| n.to_str()).ok_or(HashError {
                kind: HashErrorKind::InvalidPathError,
            }),
        )?;

        Ok(FileHash {
            content_hash,
            identity_hash,
            file_id,
        })
    }

    pub async fn hash_file_content<T>(path: T) -> Result<Blake3Hash, HashError>
    where
        T: AsRef<Path>,
    {
        let mut content = async_fs::read(path.as_ref().into()).await?;
        let mut hasher = Hasher::new();
        hasher.update(&content);
        Ok(hasher.finalize())
    }
}
