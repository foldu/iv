use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use tempfile::TempDir;
use zip::{self, ZipArchive};

#[derive(Debug, Fail)]
pub enum ZipError {
    #[fail(display = "{}", _0)]
    Io(io::Error),
    #[fail(display = "{}", _0)]
    Zip(zip::result::ZipError),
}

pub fn tmp_extract_zip<P: AsRef<Path>>(path: P) -> Result<TempDir, ZipError> {
    let path = path.as_ref();
    let mut fh = BufReader::new(File::open(&path).map_err(ZipError::Io)?);
    let mut zip = ZipArchive::new(&mut fh).map_err(ZipError::Zip)?;
    let ret = TempDir::new().map_err(ZipError::Io)?;

    for i in 0..zip.len() {
        let mut elem = zip.by_index(i).map_err(ZipError::Zip)?;
        let out_path = ret.path().join(elem.sanitized_name());
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(ZipError::Io)?;
        }

        if elem.name().ends_with('/') {
            fs::create_dir_all(&out_path).map_err(ZipError::Io)?;
        } else {
            let mut out_fh = File::create(&out_path).map_err(ZipError::Io)?;
            io::copy(&mut elem, &mut out_fh).map_err(ZipError::Io)?;
        }
    }

    Ok(ret)
}
