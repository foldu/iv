#[macro_use]
extern crate error_chain;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate gdk;
extern crate gio;
extern crate pango;
extern crate clap;

use std::process::exit;
use std::path::PathBuf;

use clap::{App, Arg};

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
        .arg(Arg::with_name("IMAGES")
                 .help("The images you want to view, skips things that are not images")
                 .required(true)
                 .multiple(true))
        .get_matches();

    let images: Vec<_> = opt.values_of_os("IMAGES")
        .unwrap()
        .map(|s| PathBuf::from(s))
        .collect();

    let app = Viewer::new(images);

    app.borrow_mut().show_all();

    gtk::main();
}
