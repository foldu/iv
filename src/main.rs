#[macro_use]
extern crate failure;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate gtk;
extern crate pango;
extern crate rayon;
extern crate walkdir;
#[macro_use]
extern crate structopt;
extern crate magic;
extern crate mime;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate directories;
extern crate serde;
extern crate tempfile;
extern crate zip;

use std::path::{Path, PathBuf};
use std::process::exit;

use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;

mod bottom_bar;
mod config;
mod extract;
mod find;
mod keys;
mod scrollable_image;
mod util;
mod viewer;

use viewer::Viewer;

fn run() -> Result<(), failure::Error> {
    gtk::init().map_err(|e| format_err!("Can't init gtk: {}", e))?;

    let opt = Opt::from_args();
    let (images, hide_status) = match opt {
        Opt {
            paths,
            hide_status,
            recursive,
        } => {
            if recursive {
                let mut ret = if paths.is_empty() {
                    vec![PathBuf::from(".")]
                } else {
                    paths.into_iter().flat_map(find::find_files_rec).collect()
                };

                // recursive dirwalking can produce a huge amount of results so why not sort it
                // in parallel
                ret.par_sort_unstable();
                (ret, hide_status)
            } else {
                let paths = if paths.is_empty() {
                    Path::new(".")
                        .read_dir()
                        .expect("Can't read current directory")
                        .filter_map(|node| node.ok())
                        .map(|node| node.path())
                        .collect()
                } else {
                    paths
                };
                (paths, hide_status)
            }
        }
    };

    let config = config::load()?;

    let app = Viewer::new(images, !hide_status, config.scrollbars, config.keymap);
    app.borrow_mut().show_all();

    gtk::main();
    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "iv")]
/// It views images
///
/// Keybinds:
///
/// q - Quit
///
/// n - Next image
///
/// p - Previous image
///
/// b - First image
///
/// e - Last image
///
/// + - Zoom in
///
/// - - Zoom out
///
/// = - Reset zoom
///
/// o - Show image in original size
///
/// w - Resize window to fit image
///
/// W - Resize window to fit original image
///
/// k - Scroll up
///
/// j - Scroll down
///
/// h - Scroll left
///
/// l - Scroll right
///
/// g - Jump to top of image
///
/// G - Jump to bottom of image
///
/// 0 - Jump left
///
/// $ - Jump right
///
/// m - Hide status bar
struct Opt {
    #[structopt(name = "PATHS", parse(from_os_str))]
    /// The things you want to view
    paths: Vec<PathBuf>,
    #[structopt(short = "s", long = "hide-status")]
    /// Hide bottom status bar
    hide_status: bool,
    #[structopt(short = "r", long = "recursive")]
    /// Recurse into directories
    recursive: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}
