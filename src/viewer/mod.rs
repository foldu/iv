mod setup;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use failure;
use gdk_pixbuf::{
    InterpType, Pixbuf, PixbufAnimation, PixbufAnimationExt, PixbufExt, PixbufRotation,
};
use gtk;
use gtk::prelude::*;
use mime;
use tempfile::TempDir;

use bottom_bar::BottomBar;
use config::{Config, WinGeom};
use extract::tmp_extract_zip;
use find;
use percent::Percent;
use ratio::*;
use scrollable_image::ScrollableImage;
use util;

pub struct Viewer {
    win: gtk::Window,
    img: ScrollableImage,
    bottom: BottomBar,
    _layout: gtk::Box,
    image_paths: Vec<PathBuf>,
    index: usize,
    cur_original_pixbuf: Option<Pixbuf>,
    cur_zoom_level: Percent,
    show_status: bool,
    tempdirs: Vec<TempDir>,
    scaling_algo: InterpType,
    initial_geom: WinGeom,
}

#[derive(Debug, Clone, Copy)]
enum Zoom {
    In,
    Out,
}

enum ImageKind {
    Animated(PixbufAnimation),
    Normal(Pixbuf),
}

fn load_image<P: AsRef<Path>>(
    path: P,
    ftype: FileType,
) -> Result<(String, ImageKind), failure::Error> {
    let path = path.as_ref();
    let path_str = if let Some(path) = path.to_str() {
        path
    } else {
        return Err(format_err!(
            "Can't decode path {:?} as UTF-8 and gtk doesn't support non UTF-8 paths",
            path
        ));
    };

    let ret = match ftype {
        FileType::AnimatedImage => ImageKind::Animated(PixbufAnimation::new_from_file(&path_str)?),
        FileType::Image => ImageKind::Normal(Pixbuf::new_from_file(&path_str)?),
        FileType::Video => {
            return Err(format_err!(
                "Can't open {}: Support for videos currently not implemented",
                path_str
            ))
        }
        _ => unreachable!(),
    };

    let filename = path.file_name().unwrap().to_str().unwrap().to_owned();

    Ok((filename, ret))
}

fn guess_file_type<P: AsRef<Path>>(path: P) -> Result<FileType, failure::Error> {
    let path = path.as_ref();
    let mime = util::mime_type_file(path)
        .map_err(|e| format_err!("Can't get mime type of {:?}: {}", path, e))?;
    if mime == mime::IMAGE_GIF {
        Ok(FileType::AnimatedImage)
    } else if mime.type_() == mime::IMAGE {
        Ok(FileType::Image)
    } else if mime.type_() == mime::VIDEO {
        Ok(FileType::Video)
    } else if mime == *util::APPLICATION_ZIP {
        Ok(FileType::Zip)
    } else {
        Err(format_err!(
            "Can't open file {:?}: Unsupported mime type: {}",
            path,
            mime
        ))
    }
}

#[derive(Debug, Clone, Copy)]
enum FileType {
    Video,
    AnimatedImage,
    Image,
    Zip,
}

impl Viewer {
    pub fn new(
        image_paths: Vec<PathBuf>,
        show_status: bool,
        config: Config,
    ) -> Rc<RefCell<Viewer>> {
        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        win.set_title("iv");

        let optimal =
            gtk_win_scale(&win, config.initial_geom.ratio, config.initial_geom.scaling).unwrap();
        win.set_default_size(optimal.0, optimal.1);
        win.set_position(gtk::WindowPosition::CenterAlways);

        // deprecated but there is no other way to set this
        // explain yourselves
        win.set_wmclass("iv", "iv");

        win.set_icon_name("emblem-photos");

        let img = ScrollableImage::new(config.scrollbars);
        let bottom = BottomBar::new(&config.bottom_format);
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.pack_start(img.as_widget(), true, true, 0);
        layout.pack_end(bottom.as_widget(), false, false, 0);

        win.add(&layout);
        let ret = Rc::new(RefCell::new(Viewer {
            win,
            img,
            bottom,
            _layout: layout,
            image_paths,
            index: 0,
            cur_original_pixbuf: None,
            cur_zoom_level: Percent::default(),
            show_status: !show_status,
            tempdirs: Vec::new(),
            scaling_algo: config.scaling_algo,
            initial_geom: config.initial_geom,
        }));

        Viewer::setup(config.keymap, &ret);

        ret
    }

    // needed because gtk::main_quit calls exit and tempdirs destructor doesn't run
    fn quit(&mut self) {
        for dir in self.tempdirs.drain(0..) {
            dir.close().expect("Can't close tempdir");
        }

        gtk::main_quit();
    }

    fn toggle_status(&mut self) {
        self.show_status = !self.show_status;
        if self.show_status {
            self.bottom.as_widget().show();
        } else {
            self.bottom.as_widget().hide();
        }
    }

    fn show_current(&mut self) -> Result<(), failure::Error> {
        let mut inner = || {
            let file_type = guess_file_type(&self.image_paths[self.index])?;
            match file_type {
                FileType::Zip => {
                    let extracted = tmp_extract_zip(&self.image_paths[self.index])?;
                    let path = extracted.path().to_owned();
                    self.tempdirs.push(extracted);

                    let new = find::find_files_rec(path);
                    let mut rest = self.image_paths.split_off(self.index);
                    let len = rest.len();
                    self.image_paths.extend(new);
                    if len > 0 {
                        rest.remove(0);
                        self.image_paths.append(&mut rest);
                    }

                    self.show_current()
                }
                FileType::Image | FileType::AnimatedImage | FileType::Video => {
                    self.show_image(file_type)
                }
            }
        };

        match inner() {
            Ok(()) => Ok(()),
            Err(e) => {
                eprintln!("{}", e);
                Err(e)
            }
        }
    }

    fn next(&mut self) {
        let tmp = self.index + 1;
        if tmp < self.image_paths.len() {
            self.index = tmp;
            if self.show_current().is_err() {
                self.image_paths.remove(self.index);
                self.index -= 1;
                self.next();
            }
        }
    }

    fn prev(&mut self) {
        if self.index != 0 {
            self.index -= 1;
            if self.show_current().is_err() {
                self.image_paths.remove(self.index);
                if self.index != 0 {
                    self.index -= 1;
                    self.prev();
                }
            }
        }
    }

    fn show_image(&mut self, ftype: FileType) -> Result<(), failure::Error> {
        match load_image(&self.image_paths[self.index], ftype) {
            Ok((filename, pixbuf)) => {
                self.win.set_title(&format!("iv - {}", &filename));

                let dims = match pixbuf {
                    ImageKind::Animated(anim) => {
                        self.img.set_from_animation(&anim);
                        self.cur_original_pixbuf = None;
                        (anim.get_width(), anim.get_height())
                    }
                    ImageKind::Normal(img) => {
                        self.img.set_from_pixbuf(&img);
                        let dims = (img.get_width(), img.get_height());
                        self.cur_original_pixbuf = Some(img);
                        dims
                    }
                };

                self.scale_to_fit_current();

                self.bottom.set_info(
                    &filename,
                    dims,
                    0,
                    self.cur_zoom_level,
                    self.index,
                    self.image_paths.len(),
                );
                Ok(())
            }
            Err(e) => {
                self.cur_original_pixbuf = None;
                Err(e)
            }
        }
    }

    fn scale_to_fit_current(&mut self) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        let alloc = self.img.get_allocation();
        let mut ratio = self.cur_zoom_level;
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            let (new_ratio, scaled) = Ratio::new(pixbuf.get_width(), pixbuf.get_height())
                .unwrap()
                .scale(alloc.width, alloc.height)
                .unwrap();
            ratio = new_ratio;
            let new_buf = pixbuf
                .scale_simple(scaled.0, scaled.1, self.scaling_algo)
                .unwrap();
            self.img.set_from_pixbuf(&new_buf);
        }
        self.set_zoom_info(ratio);
    }

    fn set_zoom_info(&mut self, percent: Percent) {
        self.cur_zoom_level = percent;
        // FIXME: USELESS ALLOC
        self.bottom.set_zoom(self.cur_zoom_level);
    }

    fn original_size(&mut self) {
        // curse you borrowck
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            self.img.set_from_pixbuf(pixbuf);
        }
        self.set_zoom_info(Percent::from(100_u32));
    }

    fn zoom(&mut self, zoomtype: Zoom) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        let percent = match zoomtype {
            Zoom::In => self
                .cur_zoom_level
                .step_next(Percent::from(25_u32), Percent::from(25_u32)),
            Zoom::Out => self
                .cur_zoom_level
                .step_prev(Percent::from(25_u32), Percent::from(25_u32)),
        };
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            let scaled = rescale(percent, pixbuf.get_width(), pixbuf.get_height()).unwrap();
            let new_buf = pixbuf
                .scale_simple(scaled.0, scaled.1, self.scaling_algo)
                .unwrap();
            self.img.set_from_pixbuf(&new_buf);
        }
        self.set_zoom_info(percent);
    }

    fn zoom_in(&mut self) {
        self.zoom(Zoom::In)
    }

    fn zoom_out(&mut self) {
        self.zoom(Zoom::Out)
    }

    fn resize_to_fit_image(&mut self) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        self.original_size();
        if let Some(ref pix) = self.cur_original_pixbuf {
            let bot_alloc = self.bottom.as_widget().get_allocation().height;
            let (img_x, img_y) = (pix.get_width(), pix.get_height());
            self.win.resize(img_x, img_y + bot_alloc);
        }
    }

    fn resize_to_fit_screen(&self) {
        let optimal = gtk_win_scale(
            &self.win,
            self.initial_geom.ratio,
            self.initial_geom.scaling,
        ).unwrap();
        self.win.resize(optimal.0, optimal.1);
    }

    fn jump_to_start(&mut self) {
        self.index = 0;
        while !self.image_paths.is_empty() {
            if self.show_current().is_err() {
                self.image_paths.remove(0);
            } else {
                break;
            }
        }
    }

    fn jump_to_end(&mut self) {
        self.index = self.image_paths.len() - 1;
        while !self.image_paths.is_empty() {
            if self.show_current().is_err() {
                let len = self.image_paths.len() - 1;
                self.image_paths.remove(len);
                self.index -= 1;
            } else {
                break;
            }
        }
    }

    fn rotate(&mut self, rot: PixbufRotation) {
        let new_orig = if let Some(ref pix) = self.cur_original_pixbuf {
            pix.rotate_simple(rot)
        } else {
            return;
        };

        self.cur_original_pixbuf = new_orig;
        self.scale_to_fit_current();
    }

    pub fn show_all(&mut self) {
        self.win.show_all();
        self.toggle_status();
        self.jump_to_start();
    }
}
