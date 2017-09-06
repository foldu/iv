use gtk;
use gtk::prelude::*;
use pango;

type Percent = f64;

pub struct BottomBar {
    boxx: gtk::Box,
    errtext: gtk::Label,
    filename: gtk::Label,
    zoom: gtk::Label,
    image_index: gtk::Label,
}

impl BottomBar {
    pub fn new() -> BottomBar {
        let errtext = gtk::Label::new(None);
        let filename = gtk::Label::new(None);
        filename.set_ellipsize(pango::EllipsizeMode::End);
        let zoom = gtk::Label::new(None);
        let image_index = gtk::Label::new(None);
        let boxx = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        boxx.pack_start(&errtext, true, false, 0);
        boxx.pack_start(&filename, false, false, 0);
        boxx.pack_start(&zoom, true, false, 0);
        boxx.pack_end(&image_index, true, false, 0);
        boxx.set_valign(gtk::Align::End);
        boxx.set_halign(gtk::Align::End);
        BottomBar {
            boxx: boxx,
            errtext: errtext,
            filename: filename,
            zoom: zoom,
            image_index: image_index,
        }
    }

    pub fn as_widget(&self) -> &gtk::Box {
        &self.boxx
    }

    pub fn set_err(&self, err: &str) {
        self.errtext.set_text(err)
    }

    pub fn set_filename(&self, text: &str) {
        self.filename.set_text(text)
    }

    pub fn set_zoom(&self, percent: Option<Percent>) {
        match percent {
            Some(percent) => self.zoom.set_text(&format!("| {:.*}%", 2, percent * 100.)),
            None => self.zoom.set_text(""),
        }
    }

    pub fn set_index(&self, index: Option<(usize, usize)>) {
        match index {
            Some((i, n)) => self.image_index.set_text(&format!("| {}/{}", i, n)),
            None => self.image_index.set_text(""),
        }
    }
}
