use std::fmt;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

use gdk_pixbuf::{self, prelude::*, Pixbuf, PixbufAnimation, PixbufLoader};
use mime;
use tempfile::TempDir;

use config::MaxFileSize;
use extract::{tmp_extract_zip, ZipError};
use find;
use humane_bytes::HumaneBytes;
use util::{self, mime_type_buf};

type Result<T> = ::std::result::Result<T, Error>;
type FileSize = u64;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(
        display = "Can't open {:?}: Cowardly refusing open {} bigger than {}", path, type_, size
    )]
    FileTooBig {
        path: PathBuf,
        type_: FileType,
        size: HumaneBytes,
    },

    #[fail(display = "IO error while reading {:?}: {}", _0, _1)]
    Io(PathBuf, #[cause] io::Error),

    #[fail(display = "{}", _0)]
    Mime(String),

    #[fail(display = "{} files currently not supported", _0)]
    Unsupported(FileType),

    #[fail(display = "Error from gdk_pixbuf: {}", _0)]
    GdkPixBuf(#[cause] gdk_pixbuf::Error),

    #[fail(display = "Failed unzipping {:?}: {}", _0, _1)]
    Unzip(PathBuf, #[cause] ZipError),
}

fn do_io<P, R, F>(path: P, f: F) -> Result<R>
where
    P: AsRef<Path>,
    F: FnOnce() -> ::std::result::Result<R, io::Error>,
{
    f().map_err(|e| Error::Io(path.as_ref().to_owned(), e))
}

pub fn load_file<P>(path: P, max_file_size: &MaxFileSize) -> Result<Loaded>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    let fh = do_io(path, || File::open(path))?;
    let metadata = do_io(path, || fh.metadata())?;

    let mut fh = BufReader::new(fh);

    // do not forget to consume nbytes
    let file_type = {
        let initial_buf = do_io(path, || fh.fill_buf())?;
        guess_file_type(&path, &initial_buf)?
    };

    let ctx = LoaderCtx {
        path,
        fh,
        file_size: metadata.len(),
    };

    match file_type {
        FileType::AnimatedImage | FileType::Image => {
            if metadata.len() > max_file_size.img.into() {
                Err(Error::FileTooBig {
                    path: path.to_owned(),
                    type_: file_type,
                    size: max_file_size.img,
                })
            } else if let FileType::AnimatedImage = file_type {
                handle_gif(ctx)
            } else {
                handle_img(ctx)
            }
        }
        FileType::Zip => {
            if metadata.len() > max_file_size.zip.into() {
                Err(Error::FileTooBig {
                    path: path.to_owned(),
                    type_: file_type,
                    size: max_file_size.zip,
                })
            } else {
                handle_zip(&ctx)
            }
        }

        FileType::Video => Err(Error::Unsupported(file_type)),
    }
}

pub enum ImageKind {
    Image(Pixbuf),
    Animated(PixbufAnimation),
}

pub enum Loaded {
    Zip {
        files: Vec<PathBuf>,
        tmp_dir: TempDir,
    },
    Image {
        size: FileSize,
        img: ImageKind,
    },
}

fn handle_gif(mut ctx: LoaderCtx) -> Result<Loaded> {
    ctx.load_pixbuf_with(|loader| loader.get_animation().unwrap())
        .map(|img| Loaded::Image {
            size: ctx.file_size,
            img: ImageKind::Animated(img),
        })
}

fn handle_img(mut ctx: LoaderCtx) -> Result<Loaded> {
    ctx.load_pixbuf_with(|loader| loader.get_pixbuf().unwrap())
        .map(|img| Loaded::Image {
            size: ctx.file_size,
            img: ImageKind::Image(img),
        })
}

fn handle_zip(ctx: &LoaderCtx) -> Result<Loaded> {
    let extracted = tmp_extract_zip(ctx.path).map_err(|e| Error::Unzip(ctx.path.to_owned(), e))?;
    let files = find::find_files_rec(extracted.path()).collect();
    Ok(Loaded::Zip {
        files,
        tmp_dir: extracted,
    })
}

#[derive(Debug)]
struct LoaderCtx<'a> {
    path: &'a Path,
    fh: BufReader<File>,
    file_size: u64,
}

impl<'a> LoaderCtx<'a> {
    fn load_pixbuf_with<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(PixbufLoader) -> T,
    {
        let mut buf = Vec::with_capacity(self.file_size as usize);
        do_io(self.path, || self.fh.read_to_end(&mut buf))?;

        let loader = PixbufLoader::new();
        loader.write(&buf).map_err(Error::GdkPixBuf)?;
        loader.close().map_err(Error::GdkPixBuf)?;
        Ok(f(loader))
    }
}

fn guess_file_type<P>(path: P, buf: &[u8]) -> Result<FileType>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let mime = mime_type_buf(buf)
        .map_err(|e| Error::Mime(format!("Can't get mime type of {:?}: {}", path, e)))?;
    if mime == mime::IMAGE_GIF {
        Ok(FileType::AnimatedImage)
    } else if mime.type_() == mime::IMAGE {
        Ok(FileType::Image)
    } else if mime.type_() == mime::VIDEO {
        Ok(FileType::Video)
    } else if mime == *util::APPLICATION_ZIP {
        Ok(FileType::Zip)
    } else {
        Err(Error::Mime(format!(
            "Can't open file {:?}: Unsupported mime type: {}",
            path, mime
        )))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Video,
    AnimatedImage,
    Image,
    Zip,
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::FileType::*;
        let s = match *self {
            Video => "video",
            AnimatedImage => "gif",
            Image => "image",
            Zip => "zip",
        };
        write!(f, "{}", s)
    }
}
