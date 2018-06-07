use std::fmt::{self, Write};

use gtk;
use gtk::prelude::*;

use humane_bytes::HumaneBytes;
use percent::Percent;
use percent_formatter::{PercentFormatBuf, PercentFormatable};

#[derive(Debug, Clone)]
struct ImageInfo {
    filename: String,
    image_index: usize,
    nimages: usize,
    dims: (i32, i32),
    file_size: String,
    zoom: Percent,
}

impl<W> PercentFormatable<W> for ImageInfo
where
    W: fmt::Write,
{
    fn try_parse(&self, rest: &str, w: &mut W) -> Result<Option<usize>, fmt::Error> {
        match rest.chars().next() {
            Some('f') => write!(w, "{}", self.filename)?,
            Some('d') => write!(w, "{}x{}", self.dims.0, self.dims.1)?,
            Some('i') => write!(w, "{}", self.image_index)?,
            Some('n') => write!(w, "{}", self.nimages)?,
            Some('s') => write!(w, "{}", self.file_size)?,
            Some('z') => write!(w, "{}", self.zoom)?,
            _ => return Ok(None),
        }

        Ok(Some(0))
    }
}

pub struct BottomBar {
    boxx: gtk::Box,
    label: gtk::Label,
    info: Option<ImageInfo>,
    formatter: PercentFormatBuf,
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
            formatter: PercentFormatBuf::new(fmt),
        }
    }

    pub fn as_widget(&self) -> &gtk::Box {
        &self.boxx
    }

    pub fn set_info(
        &mut self,
        filename: &str,
        dims: (i32, i32),
        file_size: u64,
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
                file_size: format!("{}", HumaneBytes::from(file_size)),
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
            self.render();
        }
    }
}
