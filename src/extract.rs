use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zip::{self, ZipArchive};

#[derive(Debug, Fail)]
pub enum ZipError {
    #[fail(display = "{}", _0)]
    Io(io::Error),
    #[fail(display = "Null in filename {:?}", _0)]
    NullInFilename(PathBuf),
    #[fail(display = "Path trickery in filename {:?}", _0)]
    PotentiallyMaliciousFilename(PathBuf),
    #[fail(display = "{}", _0)]
    Zip(zip::result::ZipError),
}

fn extract_path<P: AsRef<Path>>(root: P, path: &[u8]) -> Result<PathBuf, ZipError> {
    let root = root.as_ref();
    let os_path = OsStr::from_bytes(path);
    if path.contains(&b'\0') {
        return Err(ZipError::NullInFilename(PathBuf::from(os_path)));
    }

    use std::path::Component;
    let any_weird_components = Path::new(os_path).components().any(|comp| match comp {
        Component::Normal(..) => false,
        _ => true,
    });

    if any_weird_components {
        Err(ZipError::PotentiallyMaliciousFilename(PathBuf::from(
            os_path,
        )))
    } else {
        Ok(root.join(os_path))
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
