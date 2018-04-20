use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zip::{self, ZipArchive};

pub enum ZipError {
    Io(io::Error),
    NullInFilename(Vec<u8>),
    PotentiallyMaliciousFilename(Vec<u8>),
    Zip(zip::result::ZipError),
}

fn extract_path<P: AsRef<Path>>(root: P, path: &[u8]) -> Result<PathBuf, ZipError> {
    let root = root.as_ref();
    if path.contains(&b'\0') {
        return Err(ZipError::NullInFilename(Vec::from(path)));
    }

    use std::path::Component;
    let pathpath = Path::new(OsStr::from_bytes(path));
    let any_weird_components = pathpath.components().any(|comp| match comp {
        Component::Normal(..) => false,
        _ => true,
    });

    if any_weird_components {
        Err(ZipError::PotentiallyMaliciousFilename(Vec::from(path)))
    } else {
        Ok(root.join(pathpath))
    }
}

fn tmp_extract_zip<P: AsRef<Path>>(path: P) -> Result<TempDir, ZipError> {
    let path = path.as_ref();
    let mut fh = BufReader::new(File::open(&path).map_err(|e| ZipError::Io(e))?);
    let mut zip = ZipArchive::new(&mut fh).map_err(|e| ZipError::Zip(e))?;
    let ret = TempDir::new().map_err(|e| ZipError::Io(e))?;

    for i in 0..zip.len() {
        let mut elem = zip.by_index(i).map_err(|e| ZipError::Zip(e))?;
        let out_filename = extract_path(&ret, elem.name_raw())?;
        if let Some(parent) = out_filename.parent() {
            fs::create_dir_all(parent).map_err(|e| ZipError::Io(e))?;
        }
        let mut out_fh = File::create(&out_filename).map_err(|e| ZipError::Io(e))?;
        io::copy(&mut elem, &mut out_fh).map_err(|e| ZipError::Io(e))?;
    }

    Ok(ret)
}
