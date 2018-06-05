use std::fmt::Write;

use gtk;
use gtk::prelude::*;

use humane_bytes::HumaneBytes;
use percent::Percent;
use percent_formatter::{PercentFormatable, PercentFormatter};

#[derive(Debug, Clone)]
struct ImageInfo {
    filename: String,
    image_index: usize,
    nimages: usize,
    dims: (i32, i32),
    file_size: String,
    zoom: Percent,
}

impl PercentFormatable for ImageInfo {
    fn try_parse(&self, rest: &str, buf: &mut String) -> Option<usize> {
        match rest.chars().next()? {
            'f' => buf.push_str(&self.filename),
            'd' => write!(buf, "{}x{}", self.dims.0, self.dims.1).unwrap(),
            'i' => write!(buf, "{}", self.image_index).unwrap(),
            'n' => write!(buf, "{}", self.nimages).unwrap(),
            's' => buf.push_str(&self.file_size),
            'z' => write!(buf, "{}", self.zoom).unwrap(),
            _ => return None,
        }

        Some(0)
    }
}

pub struct BottomBar {
    boxx: gtk::Box,
    label: gtk::Label,
    info: Option<ImageInfo>,
    formatter: PercentFormatter,
}

impl BottomBar {
    pub fn new(fmt: &str) -> BottomBar {
        let boxx = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let label = gtk::Label::new(None);
        boxx.pack_start(&label, true, true, 0);
        boxx.set_valign(gtk::Align::End);
        boxx.set_halign(gtk::Align::End);
        BottomBar {
            boxx,
            label,
            info: None,
            formatter: PercentFormatter::new(fmt),
        }
    }

    pub fn as_widget(&self) -> &gtk::Box {
        &self.boxx
    }

    pub fn set_info(
        &mut self,
        filename: &str,
        dims: (i32, i32),
        file_size: usize,
        zoom: Percent,
        image_index: usize,
        nimages: usize,
    ) {
        let actual_index = image_index + 1;
        if let Some(ref mut info) = self.info {
            info.filename.clear();
            info.filename.push_str(filename);
            info.dims = dims;
            info.file_size.clear();
            write!(info.file_size, "{}", HumaneBytes::from(file_size)).unwrap();
            info.zoom = zoom;
            info.image_index = actual_index;
            info.nimages = nimages;
        } else {
            self.info = Some(ImageInfo {
                filename: filename.to_owned(),
                dims,
                file_size: "".to_owned(),
                zoom,
                image_index: actual_index,
                nimages,
            });
        }
        self.render();
    }

    #[inline]
    fn render(&mut self) {
        if let Some(ref info) = self.info {
            self.label.set_text(self.formatter.format(info));
        }
    }

    pub fn set_zoom(&mut self, percent: Percent) {
        if let Some(ref mut info) = self.info {
            info.zoom = percent;
        }
        self.render();
    }
}
