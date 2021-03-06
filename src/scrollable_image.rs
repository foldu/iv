use gdk_pixbuf::{Pixbuf, PixbufAnimation};
use gtk::{self, prelude::*};

pub struct ScrollableImage {
    scroll_view: gtk::ScrolledWindow,
    image: gtk::Image,
}

impl ScrollableImage {
    pub fn new(with_scrollbars: bool) -> ScrollableImage {
        let scroll_view = gtk::ScrolledWindow::new::<gtk::Adjustment, gtk::Adjustment>(None, None);
        let image = gtk::Image::new();
        scroll_view.add(&image);
        if !with_scrollbars {
            if let Some(scroll) = scroll_view.get_hscrollbar() {
                scroll.set_visible(false);
            }
            if let Some(scroll) = scroll_view.get_vscrollbar() {
                scroll.set_visible(false);
            }
        }
        ScrollableImage { scroll_view, image }
    }

    pub fn set_from_animation(&self, buf: &PixbufAnimation) {
        self.image.set_from_animation(buf)
    }

    pub fn set_from_pixbuf(&self, buf: &Pixbuf) {
        self.image.set_from_pixbuf(Some(buf))
    }

    pub fn as_widget(&self) -> &gtk::ScrolledWindow {
        &self.scroll_view
    }

    pub fn get_allocation(&self) -> gtk::Allocation {
        self.scroll_view.get_allocation()
    }

    pub fn scroll(&self, scroll: ScrollT) {
        match scroll {
            ScrollT::Up | ScrollT::Down | ScrollT::StartV | ScrollT::EndV => {
                if let Some(vadjust) = self.scroll_view.get_vadjustment() {
                    match scroll {
                        ScrollT::Up => {
                            vadjust.set_value(vadjust.get_value() - vadjust.get_step_increment())
                        }
                        ScrollT::Down => {
                            vadjust.set_value(vadjust.get_value() + vadjust.get_step_increment())
                        }
                        ScrollT::StartV => vadjust.set_value(vadjust.get_lower()),
                        ScrollT::EndV => vadjust.set_value(vadjust.get_upper()),
                        _ => (),
                    }
                }
            }
            ScrollT::Left | ScrollT::Right | ScrollT::StartH | ScrollT::EndH => {
                if let Some(hadjust) = self.scroll_view.get_hadjustment() {
                    match scroll {
                        ScrollT::Left => {
                            hadjust.set_value(hadjust.get_value() - hadjust.get_step_increment())
                        }
                        ScrollT::Right => {
                            hadjust.set_value(hadjust.get_value() + hadjust.get_step_increment())
                        }
                        ScrollT::StartH => hadjust.set_value(hadjust.get_lower()),
                        ScrollT::EndH => hadjust.set_value(hadjust.get_upper()),
                        _ => (),
                    }
                }
            }
        }
    }
}

// gtk scrolltype is missing things
#[derive(Debug, Copy, Clone)]
pub enum ScrollT {
    Up,
    Down,
    Left,
    Right,
    StartV,
    EndV,
    StartH,
    EndH,
}
