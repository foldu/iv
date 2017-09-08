#[macro_use]
extern crate error_chain;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate gdk;
extern crate gio;
extern crate pango;
extern crate clap;
extern crate walkdir;
extern crate rayon;

use std::process::exit;
use std::path::PathBuf;

use clap::{App, Arg};
use walkdir::WalkDir;
use rayon::prelude::*;

mod scrollable_image;
mod bottom_bar;
mod errors;
mod viewer;

use viewer::Viewer;

fn main() {
    if let Err(e) = gtk::init() {
        eprintln!("Can't init gtk: {}", e);
        exit(1);
    }

    let opt = App::new("iv")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("It views images")
        .arg(Arg::with_name("FILES")
                 .help("The images you want to view, skips things that are not images")
                 .required(true)
                 .multiple(true))
        .arg(Arg::with_name("recursive")
                 .help("Traverse arguments recursively (enters directories)")
                 .short("r")
                 .long("recursive"))
        .get_matches();

    let images: Vec<_> = if opt.is_present("recursive") {
        let mut ret = Vec::new();
        for arg in opt.values_of_os("FILES").unwrap() {
            let path = PathBuf::from(&arg);
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
        ret
    } else {
        opt.values_of_os("FILES")
            .unwrap()
            .map(|s| PathBuf::from(s))
            .collect()
    };

    let app = Viewer::new(images);

    app.borrow_mut().show_all();

    gtk::main();
}
