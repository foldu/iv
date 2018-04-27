use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_files_rec<P: AsRef<Path>>(root: P) -> impl Iterator<Item = PathBuf> {
    let root = root.as_ref();
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| PathBuf::from(entry.path()))
}
