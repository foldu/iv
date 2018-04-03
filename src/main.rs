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

use std::path::PathBuf;
use std::process::exit;

use rayon::prelude::*;
use structopt::StructOpt;
use walkdir::WalkDir;

mod bottom_bar;
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
                let mut ret = Vec::new();
                for path in paths {
                    if path.is_file() {
                        ret.push(path);
                    } else {
                        for entry in WalkDir::new(path) {
                            match entry {
                                Ok(ref entry) if entry.file_type().is_file() => {
                                    ret.push(PathBuf::from(entry.path()))
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // recursive dirwalking can produce a huge amount of results so why not sort it
                // in parallel
                ret.par_sort_unstable();
                (ret, hide_status)
            } else {
                (paths, hide_status)
            }
        }
    };

    let app = Viewer::new(images, !hide_status);
    app.borrow_mut().show_all();

    gtk::main();
    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "iv", about = "It views images")]
struct Opt {
    #[structopt(name = "PATHS", parse(from_os_str), help = "The things you want to view")]
    paths: Vec<PathBuf>,
    #[structopt(short = "s", long = "hide-status", help = "Hide bottom status bar")]
    hide_status: bool,
    #[structopt(short = "r", long = "recursive", help = "Recurse into directories")]
    recursive: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}
