use std::{collections::HashMap, convert::TryFrom, fmt, fs, io, path::PathBuf};

use directories::BaseDirs;
use failure::format_err;
use gdk::ModifierType;
use gdk_pixbuf::InterpType;
use gtk;
use lazy_static::lazy_static;
use serde::{
    de::{self, Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
use serde_derive::{Deserialize, Serialize};
use toml;

use crate::{
    humane_bytes::HumaneBytes,
    keys::{KeyAction, KeyMap, KeyPress},
    percent::Percent,
    ratio::Ratio,
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case", remote = "InterpType")]
#[allow(dead_code)]
enum InterpTypeDef {
    Nearest,
    Tiles,
    Bilinear,
    Hyper,
    // the fuck this shit
    __Unknown(i32),
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct WinGeom {
    pub scaling: Percent,
    pub ratio: Ratio,
}

fn def_geom() -> WinGeom {
    WinGeom {
        scaling: Percent::from(50_u32),
        ratio: Ratio::new(16, 10).unwrap(),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct MaxFileSize {
    pub zip: HumaneBytes,
    pub img: HumaneBytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub bottom_format: String,
    pub scrollbars: bool,
    #[serde(with = "InterpTypeDef")]
    pub scaling_algo: InterpType,
    pub max_file_size: MaxFileSize,
    #[serde(default = "def_geom")]
    pub initial_geom: WinGeom,
    pub keymap: KeyMap,
}

impl<'de> Deserialize<'de> for KeyPress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(KeyPressVisitor)
    }
}

impl Serialize for KeyPress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            &gtk::accelerator_name(self.0, ModifierType::empty())
                .expect("Tried to serialize invalid key combination"),
        )
    }
}

struct KeyPressVisitor;

impl<'de> Visitor<'de> for KeyPressVisitor {
    type Value = KeyPress;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a key like `Ctrl-a`")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<KeyPress, E> {
        let (keycode, _mask) = gtk::accelerator_parse(&value);
        if keycode == 0 {
            Err(E::custom(format!("Can't parse as key: {}", value)))
        } else {
            Ok(KeyPress(keycode))
        }
    }
}

macro_rules! keymap {
    ( $( $key:expr => $action:ident ),* ) => {
        {
            let mut tmp = HashMap::new();
            $(
                let (keycode, _mkey) = gtk::accelerator_parse($key);
                //if keycode == 0 {
                //    panic!("{}", $key);
                //}
                tmp.insert(KeyPress(keycode), KeyAction::$action);
             )*
                tmp
        }
    };
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bottom_format: "%d | %f | %s | %z | %i/%n".to_owned(),
            scrollbars: false,
            max_file_size: MaxFileSize {
                img: HumaneBytes::try_from("25MB").unwrap(),
                zip: HumaneBytes::try_from("256MB").unwrap(),
            },
            keymap: keymap! {
                "q" => Quit,
                "n" => Next,
                "p" => Previous,
                "equal" => ScaleToFitCurrent,
                "o" => OriginalSize,
                "w" => ResizeToFitImage,
                "W" => ResizeToFitScreen,
                "minus" => ZoomOut,
                "plus" => ZoomIn,
                "k" => ScrollUp,
                "j" => ScrollDown,
                "l" => ScrollRight,
                "h" => ScrollLeft,
                "g" => ScrollVStart,
                "G" => ScrollVEnd,
                "0" => ScrollHStart,
                "dollar" => ScrollHEnd,
                "m" => ToggleStatus,
                "b" => JumpToStart,
                "e" => JumpToEnd,
                "r" => RotateClockwise,
                "R" => RotateCounterClockwise,
                "f" => RotateUpsideDown
            },
            scaling_algo: InterpType::Bilinear,
            initial_geom: def_geom(),
        }
    }
}

lazy_static! {
    static ref CONFIG_PATH: PathBuf = BaseDirs::new().unwrap().config_dir().join("iv.toml");
}

pub fn load() -> Result<Config, failure::Error> {
    match fs::read_to_string(CONFIG_PATH.as_path()) {
        Ok(cont) => Ok(toml::from_str(&cont)?),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                write_default()
            } else {
                Err(format_err!("Can't read config: {}", e))
            }
        }
    }
}

pub fn write_default() -> Result<Config, failure::Error> {
    let ret = Config::default();
    fs::write(
        CONFIG_PATH.as_path(),
        toml::to_string_pretty(&ret).expect("Can't deserialize default config"),
    )
    .map_err(|e| format_err!("Can't write default config: {}", e))?;
    eprintln!("Default config written to {:?}", CONFIG_PATH.as_path());
    Ok(ret)
}

// FIXME: is it ok to use gtk::init() in tests?
#[test]
fn default_config_deserializeable() {
    gtk::init().unwrap();
    assert!(toml::to_string_pretty(&Config::default()).is_ok());
}
