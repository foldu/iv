use std::cell::RefCell;
use std::rc::Rc;

use gdk::enums::key;
use gtk;
use gtk::prelude::*;

use scrollable_image::ScrollT;
use viewer::Viewer;

impl Viewer {
    pub(in viewer) fn setup_keys(viewer: &Rc<RefCell<Viewer>>) {
        let clone = viewer.clone();
        viewer
            .borrow_mut()
            .win
            .connect_key_press_event(move |_, key_event| match key_event.get_keyval() {
                key::q => {
                    gtk::main_quit();
                    Inhibit(false)
                }
                key::n => {
                    clone.borrow_mut().next();
                    Inhibit(true)
                }
                key::p => {
                    clone.borrow_mut().prev();
                    Inhibit(true)
                }
                key::equal => {
                    clone.borrow_mut().scale_to_fit_current();
                    Inhibit(true)
                }
                key::o => {
                    clone.borrow_mut().original_size();
                    Inhibit(true)
                }
                key::w => {
                    clone.borrow_mut().resize_to_fit_image();
                    Inhibit(true)
                }
                key::W => {
                    clone.borrow_mut().resize_to_fit_screen();
                    Inhibit(true)
                }
                key::minus => {
                    clone.borrow_mut().zoom_out();
                    Inhibit(true)
                }
                key::plus => {
                    clone.borrow_mut().zoom_in();
                    Inhibit(true)
                }
                key::j => {
                    clone.borrow().img.scroll(ScrollT::Down);
                    Inhibit(true)
                }
                key::k => {
                    clone.borrow().img.scroll(ScrollT::Up);
                    Inhibit(true)
                }
                key::h => {
                    clone.borrow().img.scroll(ScrollT::Left);
                    Inhibit(true)
                }
                key::l => {
                    clone.borrow().img.scroll(ScrollT::Right);
                    Inhibit(true)
                }
                key::g => {
                    clone.borrow().img.scroll(ScrollT::StartV);
                    Inhibit(true)
                }
                key::G => {
                    clone.borrow().img.scroll(ScrollT::EndV);
                    Inhibit(true)
                }
                key::_0 => {
                    clone.borrow().img.scroll(ScrollT::StartH);
                    Inhibit(true)
                }
                key::dollar => {
                    clone.borrow().img.scroll(ScrollT::EndH);
                    Inhibit(true)
                }
                key::m => {
                    clone.borrow_mut().toggle_status();
                    Inhibit(true)
                }
                key::b => {
                    clone.borrow_mut().jump_to_start();
                    Inhibit(true)
                }
                key::e => {
                    clone.borrow_mut().jump_to_end();
                    Inhibit(true)
                }
                _ => Inhibit(false),
            });
    }
}
