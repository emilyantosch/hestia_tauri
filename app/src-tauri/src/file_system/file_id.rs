use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum FileId {
    Inode {
        device_id: u64,
        inode_num: u64,
    },
    Index {
        volume_serial_num: u32,
        file_index: u64,
    },
}
impl FileId {
    #[cfg(target_family = "unix")]
    pub async fn extract(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::metadata(path.as_ref())?;
        Ok(FileId::Inode {
            device_id: metadata.dev(),
            inode_num: metadata.ino(),
        })
    }

    #[cfg(target_family = "windows")]
    pub async unsafe fn extract(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        use std::{mem, os::windows::prelude::*};
        let file = Self::open_file(path)?;
        Ok(FileId::Index {
            volume_serial_num:0,file_index:0
        })
    }

    #[cfg(target_family = "windows")]
    fn open_file<P: AsRef<Path>>(path: P) -> Result<std::fs::File, Box<dyn std::error::Error>> {
        use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;

        Ok(OpenOptions::new()
            .access_mode(0)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?)
    }
}
