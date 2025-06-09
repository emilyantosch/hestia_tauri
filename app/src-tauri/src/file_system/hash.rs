use blake3::{Hash as Blake3Hash, Hasher};
use std::collections::BTreeMap;
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
    pub async fn compute(path: &Path) -> Result<FileHash, HashError> {
        let file_id = FileId::extract(path).await?;

        let content_hash = Self::hash_file_content(path).await?;

        let identity_hash = Self::hash_identity(
            &content_hash,
            &file_id,
            path.file_name().and_then(|n| n.to_str()).ok_or(HashError {
                kind: HashErrorKind::InvalidPathError,
            })?,
        )
        .await?;

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
        let content = match async_fs::read(path).await {
            Ok(content) => content,
            Err(e) => {
                return Err(HashError {
                    kind: HashErrorKind::IoError(e),
                })
            }
        };
        let mut hasher = Hasher::new();
        hasher.update(&content);
        Ok(hasher.finalize())
    }

    pub async fn hash_identity(
        content_hash: &Blake3Hash,
        file_id: &FileId,
        file_name: &str,
    ) -> Result<Blake3Hash, HashError> {
        let mut hasher = Hasher::new();

        hasher.update(content_hash.as_bytes());

        match file_id {
            FileId::Inode {
                device_id: dev,
                inode_num: ino,
            } => {
                hasher.update(&dev.to_le_bytes());
                hasher.update(&ino.to_le_bytes());
            }
            FileId::Index {
                volume_serial_num: vol_num,
                file_index: idx,
            } => {
                hasher.update(&vol_num.to_le_bytes());
                hasher.update(&idx.to_le_bytes());
            }
        }
        hasher.update(file_name.as_bytes());

        Ok(hasher.finalize())
    }
}

impl FolderHash {
    pub async fn hash(path: Path) -> Result<FolderHash, AppError> {
        let mut hasher = Hasher::new();
        let file_id = FileId::extract(path).await?;

        let structure_hash = 
    }
pub async fn hash_structure(children: &BTreeMap<String, (bool, Blake3Hash)>) -> Result<Blake3Hash, HashError> 

    }
}
