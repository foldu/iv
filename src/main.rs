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
#[macro_use]
extern crate lazy_static;

use std::path::PathBuf;
use std::process::exit;

use gtk::prelude::*;
use rayon::prelude::*;
use structopt::StructOpt;

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
                let mut ret: Vec<PathBuf> = if paths.is_empty() {
                    find::find_files_rec(".").collect()
                } else {
                    paths.into_iter().flat_map(find::find_files_rec).collect()
                };

                // recursive dirwalking can produce a huge amount of results so why not sort it
                // in parallel
                ret.par_sort_unstable();
                (ret, hide_status)
            } else {
                let paths = if paths.is_empty() {
                    find::find_files(".")
                        .map_err(|e| format_err!("Can't open current directory: {}", e))?
                        .collect()
                } else {
                    paths
                };
                (paths, hide_status)
            }
        }
    };

    match config::load() {
        Err(e) => {
            let nice_err = format!("Can't parse config: {}", e);
            eprintln!("{}", nice_err);
            let win = gtk::Window::new(gtk::WindowType::Toplevel);
            gtk::idle_add(move || {
                let dialog = gtk::MessageDialog::new(
                    Some(&win),
                    gtk::DialogFlags::empty(),
                    gtk::MessageType::Error,
                    gtk::ButtonsType::Close,
                    &nice_err,
                );
                dialog.run();
                dialog.destroy();
                gtk::main_quit();
                Continue(false)
            });
        }
        Ok(config) => {
            let app = Viewer::new(
                images,
                !hide_status,
                config.scrollbars,
                config.scaling_algo,
                config.keymap,
            );
            app.borrow_mut().show_all();
        }
    }

    gtk::main();
    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "iv")]
/// It views images
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
