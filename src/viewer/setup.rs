use std::{cell::RefCell, rc::Rc};

use gdk_pixbuf::PixbufRotation;
use gtk::prelude::*;

use crate::{
    keys::{KeyAction, KeyMap, KeyPress},
    scrollable_image::ScrollT,
    viewer::Viewer,
};
impl Viewer {
    pub(in crate::viewer) fn setup(keymap: KeyMap, viewer: &Rc<RefCell<Viewer>>) {
        let clone = viewer.clone();
        viewer
            .borrow_mut()
            .win
            .connect_key_press_event(move |_, key_event| {
                let scroll = |s| clone.borrow().img.scroll(s);
                let rot = |r| clone.borrow_mut().rotate(r);
                if let Some(action) = keymap.get(&KeyPress(key_event.get_keyval())) {
                    use self::KeyAction::*;
                    match *action {
                        Quit => clone.borrow_mut().quit(),
                        Next => clone.borrow_mut().next(),
                        Previous => clone.borrow_mut().prev(),
                        ScaleToFitCurrent => clone.borrow_mut().scale_to_fit_current(),
                        OriginalSize => clone.borrow_mut().original_size(),
                        ResizeToFitImage => clone.borrow_mut().resize_to_fit_image(),
                        ResizeToFitScreen => clone.borrow_mut().resize_to_fit_screen(),
                        ZoomOut => clone.borrow_mut().zoom_out(),
                        ZoomIn => clone.borrow_mut().zoom_in(),
                        ScrollDown => scroll(ScrollT::Down),
                        ScrollUp => scroll(ScrollT::Up),
                        ScrollLeft => scroll(ScrollT::Left),
                        ScrollRight => scroll(ScrollT::Right),
                        ScrollVStart => scroll(ScrollT::StartV),
                        ScrollVEnd => scroll(ScrollT::EndV),
                        ScrollHStart => scroll(ScrollT::StartH),
                        ScrollHEnd => scroll(ScrollT::EndH),
                        ToggleStatus => clone.borrow_mut().toggle_status(),
                        JumpToStart => clone.borrow_mut().jump_to_start(),
                        JumpToEnd => clone.borrow_mut().jump_to_end(),
                        RotateClockwise => rot(PixbufRotation::Clockwise),
                        RotateCounterClockwise => rot(PixbufRotation::Counterclockwise),
                        RotateUpsideDown => rot(PixbufRotation::Upsidedown),
                    };
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            });

        let clone = viewer.clone();
        viewer.borrow_mut().win.connect_delete_event(move |_, _| {
            clone.borrow_mut().quit();
            Inhibit(false)
        });
    }
}
