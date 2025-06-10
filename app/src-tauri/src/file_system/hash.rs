use async_recursion::async_recursion;
use blake3::{Hash as Blake3Hash, Hasher};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;

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
    pub async fn hash(path: &Path) -> Result<FileHash, AppError> {
        let file_id = FileId::extract(path).await?;

        let content_hash = Self::hash_file_content(path).await?;

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(file_name) => Ok(file_name),
            None => Err(HashError::new(
                HashErrorKind::InvalidPathError,
                format!("Could not find path: {:?}", path),
            )),
        }?;

        let identity_hash = Self::hash_identity(&content_hash, &file_id, file_name).await?;

        Ok(FileHash {
            content_hash,
            identity_hash,
            file_id,
        })
    }

    pub async fn hash_file_content<T>(path: T) -> Result<Blake3Hash, HashError>
    where
        T: AsRef<Path> + std::fmt::Debug + Clone,
    {
        let content = match async_fs::read(path.clone()).await {
            Ok(content) => content,
            Err(e) => {
                return Err(HashError::with_source(
                    HashErrorKind::IoError,
                    format!("Could not find path: {:?}", path),
                    e,
                ))
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
    #[async_recursion]
    pub async fn hash(path: &Path) -> Result<FolderHash, AppError> {
        let file_id = FileId::extract(path).await?;

        let (structure_hash, content_hash) = Self::_hash_folder(path).await?;

        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(folder_name) => Ok(folder_name),
            None => Err(HashError::new(
                HashErrorKind::InvalidPathError,
                format!("Could not find path: {:?}", path),
            )),
        }?;

        let identity_hash =
            Self::hash_identity(&structure_hash, &content_hash, &file_id, folder_name).await?;

        Ok(FolderHash {
            structure_hash,
            content_hash,
            identity_hash,
            file_id,
        })
    }

    #[async_recursion]
    async fn _hash_folder(path: &Path) -> Result<(Blake3Hash, Blake3Hash), HashError> {
        let mut entries = async_fs::read_dir(path).await.map_err(|e| {
            HashError::with_source(
                HashErrorKind::IoError,
                format!("Async fs could not read directory at path: {:?}", path),
                e,
            )
        })?;
        let mut children: BTreeMap<String, (bool, Blake3Hash)> = BTreeMap::new();
        let mut all_content_hashes: Vec<Blake3Hash> = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let entry_name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                let file_hash = FileHash::hash(&entry_path).await.map_err(|e| {
                    HashError::with_source(
                        HashErrorKind::IoError,
                        format!(
                            "The hash function failed at a lower level: {:?}",
                            e.to_string()
                        ),
                        e,
                    )
                })?;
                children.insert(entry_name, (false, file_hash.content_hash));
                all_content_hashes.push(file_hash.content_hash);
            } else if metadata.is_dir() {
                let folder_hash = FolderHash::hash(&entry_path).await.map_err(|e| {
                    HashError::with_source(
                        HashErrorKind::IoError,
                        format!(
                            "The hash function failed at a lower level: {:?}",
                            e.to_string()
                        ),
                        e,
                    )
                })?;
                children.insert(entry_name, (true, folder_hash.content_hash));
                all_content_hashes.push(folder_hash.content_hash);
            }
        }

        // Hash the immediate structure of the directory
        let structure_hash = Self::hash_structure(&children).await?;

        // Hash the content of that directory
        let content_hash = Self::hash_content(&mut all_content_hashes).await?;

        Ok((structure_hash, content_hash))
    }

    async fn hash_structure(
        children: &BTreeMap<String, (bool, Blake3Hash)>,
    ) -> Result<Blake3Hash, HashError> {
        let mut hasher = Hasher::new();

        for (name, (is_dir, _)) in children {
            hasher.update(name.as_bytes());
            hasher.update(&[if *is_dir { 1u8 } else { 0u8 }]);
        }
        Ok(hasher.finalize())
    }

    async fn hash_content(content_hashes: &mut [Blake3Hash]) -> Result<Blake3Hash, HashError> {
        let mut hasher = Hasher::new();

        content_hashes.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for hash in content_hashes {
            hasher.update(hash.as_bytes());
        }

        Ok(hasher.finalize())
    }

    async fn hash_identity(
        structure_hash: &Blake3Hash,
        content_hash: &Blake3Hash,
        file_id: &FileId,
        folder_name: &str,
    ) -> Result<Blake3Hash, HashError> {
        let mut hasher = Hasher::new();

        hasher.update(structure_hash.as_bytes());
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
        hasher.update(folder_name.as_bytes());

        Ok(hasher.finalize())
    }
}
