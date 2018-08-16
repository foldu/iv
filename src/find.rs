use std::io;
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

pub fn find_files<P: AsRef<Path>>(root: P) -> Result<impl Iterator<Item = PathBuf>, io::Error> {
    let root = root.as_ref();
    root.read_dir().map(|entries| {
        entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                Some((entry.file_type().ok()?, entry))
            }).filter(|(t, _)| t.is_file())
            .map(|(_, entry)| entry.path())
    })
}
