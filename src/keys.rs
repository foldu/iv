use std::collections::HashMap;

use gdk::ModifierType;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct KeyPress(pub u32, pub ModifierType);

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Quit,
    Next,
    Previous,
    ScaleToFitCurrent,
    OriginalSize,
    ResizeToFitImage,
    ResizeToFitScreen,
    ZoomOut,
    ZoomIn,
    ScrollDown,
    ScrollUp,
    ScrollLeft,
    ScrollRight,
    ScrollVStart,
    ScrollVEnd,
    ScrollHStart,
    ScrollHEnd,
    ToggleStatus,
    JumpToStart,
    JumpToEnd,
}

pub type KeyMap = HashMap<KeyPress, KeyAction>;
