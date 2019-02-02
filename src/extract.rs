use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::Path,
};

use failure::Fail;
use tempfile::TempDir;
use zip::{self, ZipArchive};

#[derive(Debug, Fail)]
pub enum ZipError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    Zip(#[cause] zip::result::ZipError),
}

impl From<io::Error> for ZipError {
    fn from(e: io::Error) -> Self {
        ZipError::Io(e)
    }
}

impl From<zip::result::ZipError> for ZipError {
    fn from(e: zip::result::ZipError) -> Self {
        ZipError::Zip(e)
    }
}

pub fn tmp_extract_zip<P: AsRef<Path>>(path: P) -> Result<TempDir, ZipError> {
    let path = path.as_ref();
    let mut fh = BufReader::new(File::open(&path)?);
    let mut zip = ZipArchive::new(&mut fh)?;
    let ret = TempDir::new()?;

    for i in 0..zip.len() {
        let mut elem = zip.by_index(i)?;
        let out_path = ret.path().join(elem.sanitized_name());
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if elem.name().ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            let mut out_fh = File::create(&out_path)?;
            io::copy(&mut elem, &mut out_fh)?;
        }
    }

    Ok(ret)
}
