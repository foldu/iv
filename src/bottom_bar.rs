use gtk;
use gtk::prelude::*;

type Percent = f64;

pub struct BottomBar {
    boxx: gtk::Box,
    errtext: gtk::Label,
    filename: gtk::Label,
    zoom: gtk::Label,
}

impl BottomBar {
    pub fn new() -> BottomBar {
        let errtext = gtk::Label::new(None);
        let filename = gtk::Label::new(None);
        let zoom = gtk::Label::new(None);
        let boxx = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        boxx.pack_start(&errtext, true, false, 0);
        boxx.pack_start(&filename, true, false, 0);
        boxx.pack_end(&zoom, true, false, 0);
        boxx.set_valign(gtk::Align::End);
        boxx.set_halign(gtk::Align::End);
        BottomBar {
            boxx: boxx,
            errtext: errtext,
            filename: filename,
            zoom: zoom,
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

    pub fn set_zoom(&self, percent: Percent) {
        self.zoom.set_text(&format!("{:.*}%", 2, percent * 100.))
    }
}
