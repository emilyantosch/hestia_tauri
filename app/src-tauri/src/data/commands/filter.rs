use std::path::PathBuf;

use crate::data::{folder::Folder, tag::Tag};

#[derive(Debug)]
pub struct Filter {
    pub tag_filter: Option<TagFilter>,
    pub folder_filter: Option<FolderFilter>,
}

#[derive(Debug)]
pub struct TagFilter {
    pub tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct FolderFilter {
    pub folders: Vec<PathBuf>,
}
