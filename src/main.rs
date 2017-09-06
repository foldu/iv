#[macro_use]
extern crate error_chain;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate gdk;
extern crate gio;

use std::process::exit;
use std::path::{Path, PathBuf};
use std::env::args_os;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gdk_pixbuf::{Pixbuf, PixbufAnimation};
use gdk::enums::key;

mod scrollable_image;
mod bottom_bar;
mod errors;

use scrollable_image::{ScrollableImage, ScrollT};
use bottom_bar::BottomBar;
use errors::*;

type Percent = f64;

enum ImageKind {
    Animated(PixbufAnimation),
    Normal(Pixbuf),
}

fn load_image<P: AsRef<Path>>(path: P) -> Result<(String, ImageKind)> {
    let path_str = if let Some(path) = path.as_ref().to_str() {
        path
    } else {
        bail!(format!("Can't decode \"{:?}\" as UTF-8",
                      path.as_ref().to_string_lossy()));
    };

    let (mime_guess, result_certain) = gio::content_type_guess(path_str, &[]);

    let ret = if mime_guess == "image/gif" {
        ImageKind::Animated(PixbufAnimation::new_from_file(&path_str)?)
    } else {
        ImageKind::Normal(Pixbuf::new_from_file(&path_str)?)
    };

    let filename = path.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    Ok((filename, ret))
}

type V2 = (i32, i32);

fn aspect_ratio_zoom(orig: V2, ratio: Percent) -> V2 {
    ((orig.0 as f64 * ratio).floor() as i32, (orig.1 as f64 * ratio).floor() as i32)
}

fn scale_with_aspect_ratio(orig: V2, scale_to: V2) -> (Percent, V2) {
    let ratio = f64::min(scale_to.0 as f64 / orig.0 as f64,
                         scale_to.1 as f64 / orig.1 as f64);
    let scaled = aspect_ratio_zoom(orig, ratio);
    (ratio, scaled)
}

enum Zoom {
    In,
    Out,
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

struct Viewer {
    win: gtk::Window,
    img: ScrollableImage,
    bottom: BottomBar,
    layout: gtk::Box,
    image_paths: Vec<PathBuf>,
    index: usize,
    cur_original_pixbuf: Option<Pixbuf>,
    cur_ratio: Percent,
}

impl Viewer {
    pub fn new(image_paths: Vec<PathBuf>) -> Rc<RefCell<Viewer>> {
        let win = gtk::Window::new(gtk::WindowType::Toplevel);
        win.set_title("iv");
        if let Some(scr) = win.get_screen() {
            win.set_default_size(640, 480);
        }
        win.set_position(gtk::WindowPosition::CenterAlways);
        win.connect_delete_event(|_, _| {
                                     gtk::main_quit();
                                     Inhibit(false)
                                 });
        // deprecated but there is no other way to set this
        // explain yourselves
        win.set_wmclass("iv", "iv");


        let img = ScrollableImage::new();
        let bottom = BottomBar::new();
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
        layout.pack_start(img.as_widget(), true, true, 0);
        layout.pack_end(bottom.as_widget(), false, false, 0);


        win.add(&layout);
        let ret = Rc::new(RefCell::new(Viewer {
                                           win: win,
                                           img: img,
                                           bottom: bottom,
                                           layout: layout,
                                           image_paths: image_paths,
                                           index: 0,
                                           cur_original_pixbuf: None,
                                           cur_ratio: 0.,
                                       }));
        let ret_conn = ret.clone();

        ret.borrow_mut().win.connect_key_press_event(move |_, key_event| {
            match key_event.get_keyval() {
                key::q => {
                    gtk::main_quit();
                    Inhibit(false)

                }
                key::n => {
                    ret_conn.borrow_mut().next();
                    Inhibit(true)
                }
                key::p => {
                    ret_conn.borrow_mut().prev();
                    Inhibit(true)
                }
                key::equal => {
                    ret_conn.borrow_mut().scale_to_fit_current();
                    Inhibit(true)
                }
                key::o => {
                    ret_conn.borrow_mut().original_size();
                    Inhibit(true)
                }
                key::minus => {
                    ret_conn.borrow_mut().zoom_out();
                    Inhibit(true)
                }
                key::plus => {
                    ret_conn.borrow_mut().zoom_in();
                    Inhibit(true)
                }
                key::j => {
                    ret_conn.borrow().img.scroll(ScrollT::Down);
                    Inhibit(true)
                }
                key::k => {
                    ret_conn.borrow().img.scroll(ScrollT::Up);
                    Inhibit(true)
                }
                key::h => {
                    ret_conn.borrow().img.scroll(ScrollT::Left);
                    Inhibit(true)
                }
                key::l => {
                    ret_conn.borrow().img.scroll(ScrollT::Right);
                    Inhibit(true)
                }
                key::g => {
                    ret_conn.borrow().img.scroll(ScrollT::StartV);
                    Inhibit(true)
                }
                key::G => {
                    ret_conn.borrow().img.scroll(ScrollT::EndV);
                    Inhibit(true)
                }
                key::_0 => {
                    ret_conn.borrow().img.scroll(ScrollT::StartH);
                    Inhibit(true)
                }
                key::dollar => {
                    ret_conn.borrow().img.scroll(ScrollT::EndH);
                    Inhibit(true)
                }
                _ => Inhibit(false),
            }
        });

        ret
    }

    fn next(&mut self) {
        let tmp = self.index + 1;
        if tmp < self.image_paths.len() {
            self.index = tmp;
            if self.show_image().is_err() {
                self.image_paths.remove(self.index);
                self.next();
            }
        }
    }

    fn prev(&mut self) {
        if self.index != 0 {
            self.index -= 1;
            let _ = self.show_image();
        }
    }

    fn show_image(&mut self) -> Result<()> {
        match load_image(&self.image_paths[self.index]) {
            Ok((filename, pixbuf)) => {
                self.win.set_title(&format!("iv - {}", &filename));
                self.bottom.set_filename(&filename);

                match pixbuf {
                    ImageKind::Animated(anim) => {
                        self.img.set_from_animation(&anim);
                        self.cur_original_pixbuf = None;
                        self.bottom.set_zoom(None);
                    }
                    ImageKind::Normal(img) => {
                        self.img.set_from_pixbuf(&img);
                        self.cur_original_pixbuf = Some(img);
                    }
                }
                self.bottom.set_err("");
                self.scale_to_fit_current();
                Ok(())
            }
            Err(e) => {
                self.cur_original_pixbuf = None;
                // self.bottom.set_filename("");
                // self.bottom.set_zoom(0.);
                // self.bottom.set_err(e.description());
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
            let (new_ratio, scaled) = scale_with_aspect_ratio((pixbuf.get_width(),
                                                               pixbuf.get_height()),
                                                              (alloc.width, alloc.height));
            ratio = new_ratio;
            let new_buf = pixbuf.scale_simple(scaled.0, scaled.1, gdk_pixbuf::InterpType::Bilinear)
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
            let new_buf = pixbuf.scale_simple(scaled.0, scaled.1, gdk_pixbuf::InterpType::Bilinear)
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


    pub fn show_all(&mut self) {
        self.win.show_all();
        if self.image_paths.len() != 0 {
            self.show_image();
        } else {
            self.bottom.set_err("No images found");
        }
    }
}

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
