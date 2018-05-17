use gtk;
use gtk::prelude::*;
use pango;

use percent::Percent;

pub struct BottomBar {
    boxx: gtk::Box,
    resolution: gtk::Label,
    filename: gtk::Label,
    zoom: gtk::Label,
    image_index: gtk::Label,
}

impl BottomBar {
    pub fn new() -> BottomBar {
        let filename = gtk::Label::new(None);
        filename.set_ellipsize(pango::EllipsizeMode::End);
        let zoom = gtk::Label::new(None);
        let image_index = gtk::Label::new(None);
        let resolution = gtk::Label::new(None);
        let boxx = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        boxx.pack_start(&resolution, false, false, 0);
        boxx.pack_start(&filename, false, false, 0);
        boxx.pack_start(&zoom, true, false, 0);
        boxx.pack_end(&image_index, true, false, 0);
        boxx.set_valign(gtk::Align::End);
        boxx.set_halign(gtk::Align::End);
        BottomBar {
            boxx,
            resolution,
            filename,
            zoom,
            image_index,
        }
    }

    pub fn as_widget(&self) -> &gtk::Box {
        &self.boxx
    }

    pub fn set_info(&self, filename: &str, dims: (i32, i32)) {
        self.filename.set_text(&format!("| {}", filename));
        self.resolution.set_text(&format!("{}x{}", dims.0, dims.1));
    }

    pub fn set_zoom(&self, percent: Option<Percent>) {
        match percent {
            Some(percent) => self.zoom.set_text(&format!("| {}", percent)),
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
