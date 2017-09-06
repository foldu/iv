#[macro_use]
extern crate error_chain;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate gdk;
extern crate gio;
extern crate pango;

use std::process::exit;
use std::env::args_os;
use std::path::PathBuf;

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

    let images: Vec<_> = args_os().skip(1).map(|s| PathBuf::from(s)).collect();

    let app = Viewer::new(images);

    app.borrow_mut().show_all();

    gtk::main();
}
