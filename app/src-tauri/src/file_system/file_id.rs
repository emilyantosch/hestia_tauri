use std::path::Path;

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
    pub async fn extract(path: impl AsRef<Path>) -> Result<Self, crate::errors::HestiaErrors> {
        use std::os::unix::fs::MetadataExt;
        let metadata = std::fs::metadata(path.as_ref())?;
        Ok(FileId::Inode {
            device_id: metadata.dev(),
            inode_num: metadata.ino(),
        })
    }

    #[cfg(target_family = "windows")]
    pub async unsafe fn get_file_win_id(path: &PathBuf) -> Result<Self> {
        use std::{mem, os::windows::prelude::*};
        let file = open_file(path)?;
    }

    #[cfg(target_family = "windows")]
    fn open_file<P: AsRef<Path>>(path: P) -> Result<fs::File> {
        use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};
        use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;

        OpenOptions::new()
            .access_mode(0)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)
    }
}
