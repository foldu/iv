use std::cell::RefCell;
use std::rc::Rc;

use gtk;
use gtk::prelude::*;

use keys::{KeyAction, KeyMap, KeyPress};
use scrollable_image::ScrollT;
use viewer::Viewer;
impl Viewer {
    pub(in viewer) fn setup_keys(keymap: KeyMap, viewer: &Rc<RefCell<Viewer>>) {
        let clone = viewer.clone();
        viewer
            .borrow_mut()
            .win
            .connect_key_press_event(move |_, key_event| {
                let scroll = |s| clone.borrow().img.scroll(s);
                if let Some(action) = keymap.get(&KeyPress(key_event.get_keyval())) {
                    use self::KeyAction::*;
                    match *action {
                        Quit => gtk::main_quit(),
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
                    };
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            });
    }
}
