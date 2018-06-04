use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;

use directories::BaseDirs;
use failure;
use gdk::ModifierType;
use gdk_pixbuf::InterpType;
use gtk;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use toml;

use keys::{KeyAction, KeyMap, KeyPress};
use percent::Percent;
use ratio::Ratio;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub scrollbars: bool,
    #[serde(with = "InterpTypeDef")]
    pub scaling_algo: InterpType,
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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
            scrollbars: false,
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
                "e" => JumpToEnd
            },
            scaling_algo: InterpType::Bilinear,
            initial_geom: def_geom(),
        }
    }
}

pub fn load() -> Result<Config, failure::Error> {
    let path = BaseDirs::new().config_dir().join("iv.toml");
    match fs::read_to_string(&path) {
        Ok(cont) => Ok(toml::from_str(&cont)?),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                let ret = Config::default();
                fs::write(
                    &path,
                    toml::to_string_pretty(&ret).expect("Can't deserialize default config"),
                )?;
                eprintln!("Default config written to {:?}", path);
                Ok(ret)
            } else {
                Err(format_err!("Can't read config: {}", e))
            }
        }
    }
}
