mod setup;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use failure;
use gdk::ScreenExt;
use gdk_pixbuf;
use gdk_pixbuf::{Pixbuf, PixbufAnimation, PixbufAnimationExt, PixbufExt};
use gtk;
use gtk::prelude::*;
use mime;
use tempfile::TempDir;

use bottom_bar::BottomBar;
use extract::tmp_extract_zip;
use find;
use keys::KeyMap;
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
    cur_ratio: Percent,
    show_status: bool,
    tempdirs: Vec<TempDir>,
}

type Percent = f64;

enum ImageKind {
    Animated(PixbufAnimation),
    Normal(Pixbuf),
}

fn load_image<P: AsRef<Path>>(path: P) -> Result<(String, ImageKind), failure::Error> {
    let path = path.as_ref();
    let path_str = if let Some(path) = path.to_str() {
        path
    } else {
        return Err(format_err!("Can't decode path {:?} as UTF-8", path));
    };

    let mime = util::mime_type_file(&path_str)
        .map_err(|e| format_err!("Can't get mime type of file {:?}: {}", path, e))?;

    let ret = if mime == mime::IMAGE_GIF {
        ImageKind::Animated(PixbufAnimation::new_from_file(&path_str)?)
    } else if mime.type_() == mime::IMAGE {
        ImageKind::Normal(Pixbuf::new_from_file(&path_str)?)
    } else {
        return Err(format_err!(
            "Can't open file {:?}: Can't open files with mime type {}",
            path,
            mime
        ));
    };

    let filename = path.file_name()
        .ok_or_else(|| format_err!("Missing filename in path {:?}", path))
        .map(|filename| filename.to_str().unwrap().to_owned())?;

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
        Err(format_err!("Unsupported mime type: {}", mime))
    }
}

type V2 = (i32, i32);

fn aspect_ratio_zoom(orig: V2, ratio: Percent) -> V2 {
    (
        (orig.0 as f64 * ratio).floor() as i32,
        (orig.1 as f64 * ratio).floor() as i32,
    )
}

fn scale_with_aspect_ratio(orig: V2, scale_to: V2) -> (Percent, V2) {
    let ratio = f64::min(
        scale_to.0 as f64 / orig.0 as f64,
        scale_to.1 as f64 / orig.1 as f64,
    );
    let scaled = aspect_ratio_zoom(orig, ratio);
    (ratio, scaled)
}

fn optimal_16_10_win_size(win: &gtk::Window) -> V2 {
    let scr = win.get_screen().expect("Can't get display size");
    let dims = scr.get_monitor_geometry(scr.get_number());
    let (_, optimal) = scale_with_aspect_ratio((16, 10), (dims.width / 2, dims.height / 2));
    optimal
}

enum Zoom {
    In,
    Out,
}

enum FileType {
    Video,
    AnimatedImage,
    Image,
    Zip,
}

fn next_zoom_stage(mut percent: Percent, zoom_opt: Zoom) -> Percent {
    match zoom_opt {
        Zoom::In => {
            percent += 0.25;
        }
        Zoom::Out => {
            percent -= 0.25;
        }
    };

    match (percent / 0.25).round() * 0.25 {
        x if x < 0.25 => 0.25,
        x => x,
    }
}

impl Viewer {
    pub fn new(
        image_paths: Vec<PathBuf>,
        show_status: bool,
        with_scrollbars: bool,
        keymap: KeyMap,
    ) -> Rc<RefCell<Viewer>> {
        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        win.set_title("iv");

        let optimal = optimal_16_10_win_size(&win);
        win.set_default_size(optimal.0, optimal.1);
        win.set_position(gtk::WindowPosition::CenterAlways);

        // deprecated but there is no other way to set this
        // explain yourselves
        win.set_wmclass("iv", "iv");

        win.set_icon_name("emblem-photos");

        let img = ScrollableImage::new(with_scrollbars);
        let bottom = BottomBar::new();
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.pack_start(img.as_widget(), true, true, 0);
        layout.pack_end(bottom.as_widget(), false, false, 0);

        win.add(&layout);
        let ret = Rc::new(RefCell::new(Viewer {
            win: win,
            img: img,
            bottom: bottom,
            _layout: layout,
            image_paths: image_paths,
            index: 0,
            cur_original_pixbuf: None,
            cur_ratio: 0.,
            show_status: !show_status,
            tempdirs: Vec::new(),
        }));

        Viewer::setup(keymap, &ret);

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
                FileType::Image | FileType::AnimatedImage | FileType::Video => self.show_image(),
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
        } else {
            self.show_current().expect("Wrong");
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
        } else {
            self.show_current().expect("This shouldn't happen");
        }
    }

    fn show_image(&mut self) -> Result<(), failure::Error> {
        match load_image(&self.image_paths[self.index]) {
            Ok((filename, pixbuf)) => {
                self.win.set_title(&format!("iv - {}", &filename));

                let dims = match pixbuf {
                    ImageKind::Animated(anim) => {
                        self.img.set_from_animation(&anim);
                        self.cur_original_pixbuf = None;
                        self.bottom.set_zoom(None);
                        (anim.get_width(), anim.get_height())
                    }
                    ImageKind::Normal(img) => {
                        self.img.set_from_pixbuf(&img);
                        let dims = (img.get_width(), img.get_height());
                        self.cur_original_pixbuf = Some(img);
                        dims
                    }
                };

                self.bottom.set_info(&filename, dims);

                if self.image_paths.len() > 1 {
                    self.bottom
                        .set_index(Some((self.index + 1, self.image_paths.len())));
                } else {
                    self.bottom.set_index(None);
                }
                self.scale_to_fit_current();
                Ok(())
            }
            Err(e) => {
                self.cur_original_pixbuf = None;
                self.bottom.set_index(None);
                Err(e)
            }
        }
    }

    fn scale_to_fit_current(&mut self) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        let alloc = self.img.get_allocation();
        let mut ratio = self.cur_ratio;
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            let (new_ratio, scaled) = scale_with_aspect_ratio(
                (pixbuf.get_width(), pixbuf.get_height()),
                (alloc.width, alloc.height),
            );
            ratio = new_ratio;
            let new_buf = pixbuf
                .scale_simple(scaled.0, scaled.1, gdk_pixbuf::InterpType::Bilinear)
                .unwrap();
            self.img.set_from_pixbuf(&new_buf);
        }
        self.set_zoom_info(ratio);
    }

    fn set_zoom_info(&mut self, percent: Percent) {
        self.cur_ratio = percent;
        self.bottom.set_zoom(Some(percent));
    }

    fn original_size(&mut self) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            self.img.set_from_pixbuf(pixbuf);
        }
        self.set_zoom_info(1.);
    }

    fn zoom(&mut self, zoomtype: Zoom) {
        if self.cur_original_pixbuf.is_none() {
            return;
        }
        let ratio = next_zoom_stage(self.cur_ratio, zoomtype);
        if let Some(ref pixbuf) = self.cur_original_pixbuf {
            let scaled = aspect_ratio_zoom((pixbuf.get_width(), pixbuf.get_height()), ratio);
            let new_buf = pixbuf
                .scale_simple(scaled.0, scaled.1, gdk_pixbuf::InterpType::Bilinear)
                .unwrap();
            self.img.set_from_pixbuf(&new_buf);
        }
        self.set_zoom_info(ratio);
    }

    fn zoom_in(&mut self) {
        self.zoom(Zoom::In)
    }

    fn zoom_out(&mut self) {
        self.zoom(Zoom::Out)
    }

    fn resize_to_fit_image(&mut self) {
        if let None = self.cur_original_pixbuf {
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
        let optimal = optimal_16_10_win_size(&self.win);
        self.win.resize(optimal.0, optimal.1);
    }

    fn jump_to_start(&mut self) {
        self.index = 0;
        while self.image_paths.len() > 0 {
            if let Err(_) = self.show_current() {
                self.image_paths.remove(0);
            } else {
                break;
            }
        }
    }

    fn jump_to_end(&mut self) {
        self.index = self.image_paths.len() - 1;
        while self.image_paths.len() > 0 {
            if let Err(_) = self.show_current() {
                let len = self.image_paths.len() - 1;
                self.image_paths.remove(len);
                self.index -= 1;
            } else {
                break;
            }
        }
    }

    pub fn show_all(&mut self) {
        self.win.show_all();
        self.toggle_status();
        self.jump_to_start();
    }
}
