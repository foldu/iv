#![feature(try_from, nll)]
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
extern crate indexmap;
extern crate noisy_float;
extern crate num;
#[macro_use]
extern crate nom;

use std::path::PathBuf;
use std::process::exit;

use gtk::prelude::*;
use rayon::prelude::*;
use structopt::StructOpt;

mod bottom_bar;
mod config;
mod extract;
mod find;
mod humane_bytes;
mod keys;
mod parse;
#[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
mod percent;
mod percent_formatter;
mod ratio;
mod scrollable_image;
mod util;
mod viewer;

use viewer::Viewer;

fn run() -> Result<(), failure::Error> {
    let opt = Opt::from_args();

    gtk::init().map_err(|e| format_err!("Can't init gtk: {}", e))?;

    if opt.write_default {
        config::write_default()?;
        return Ok(());
    }

    match config::load() {
        Err(e) => {
            let nice_err = format!("Can't parse config: {}", e);
            eprintln!("{}", nice_err);
            display_error_dialog(nice_err);
        }
        Ok(config) => {
            let (images, hide_status) = opt_to_viewer_params(opt)?;
            let app = Viewer::new(images, !hide_status, config);
            app.borrow_mut().show_all();
        }
    }

    gtk::main();
    Ok(())
}

fn display_error_dialog(nice_err: String) {
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

#[inline]
fn opt_to_viewer_params(
    Opt {
        hide_status,
        recursive,
        paths,
        ..
    }: Opt,
) -> Result<(Vec<PathBuf>, bool), failure::Error> {
    if recursive {
        let mut ret: Vec<PathBuf> = if paths.is_empty() {
            find::find_files_rec(".").collect()
        } else {
            paths.into_iter().flat_map(find::find_files_rec).collect()
        };

        // recursive dirwalking can produce a huge amount of results so why not sort it
        // in parallel
        ret.par_sort_unstable();
        Ok((ret, hide_status))
    } else {
        let paths = if paths.is_empty() {
            find::find_files(".")
                .map_err(|e| format_err!("Can't open current directory: {}", e))?
                .collect()
        } else {
            paths
        };
        Ok((paths, hide_status))
    }
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
    #[structopt(long = "write-default")]
    /// Just write the default config, clobbering the old one
    write_default: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        exit(1);
    }
}
