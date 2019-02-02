use std::convert::TryFrom;
use std::fmt;
use std::ops;

use gdk::ScreenExt;
use gtk::{self, prelude::*};
use num::{FromPrimitive, ToPrimitive};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use percent::Percent;

/// A ratio. Can be used for more than just correct aspect ratio transforms.
#[derive(Debug, Copy, Clone)]
pub struct Ratio(f64, f64);

impl<'a> TryFrom<&'a str> for Ratio {
    type Error = &'static str;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        (|| {
            let mut it = s.splitn(2, 'x');
            let parse = |x: &mut Iterator<Item = &str>| x.next().and_then(|x| x.parse().ok());
            let (a, b) = (parse(&mut it)?, parse(&mut it)?);
            if it.next().is_some() {
                None
            } else {
                Some(Ratio(a, b))
            }
        })()
        .ok_or("Expecting f64xf64")
    }
}

impl Serialize for Ratio {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let out = format!("{}x{}", self.0, self.1);
        serializer.serialize_str(&out)
    }
}

impl<'de> Deserialize<'de> for Ratio {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RatioVisitor;
        impl<'de> de::Visitor<'de> for RatioVisitor {
            type Value = Ratio;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ratio")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ratio::try_from(value).map_err(|e| E::custom(e.to_string()))
            }
        }

        deserializer.deserialize_str(RatioVisitor)
    }
}

/// Rescale operations that's guaranteed to work
impl ops::Mul<Percent> for Ratio {
    type Output = Self;
    fn mul(self, rhs: Percent) -> Self {
        let res = rescale(rhs, self.0, self.1).unwrap();
        Ratio(res.0, res.1)
    }
}

impl Ratio {
    pub fn new<T: ToPrimitive + Copy>(a: T, b: T) -> Option<Ratio> {
        Some(Ratio(a.to_f64()?, b.to_f64()?))
    }

    pub fn scale<T: FromPrimitive + ToPrimitive + Copy>(
        &self,
        a: T,
        b: T,
    ) -> Option<(Percent, (T, T))> {
        let (a_f, b_f) = (a.to_f64()?, b.to_f64()?);
        let ratio = f64::min(a_f / self.0, b_f / self.1);
        let ratio = Percent::try_from(ratio).ok()?;
        let scaled = rescale(ratio, T::from_f64(self.0)?, T::from_f64(self.1)?)?;
        Some((ratio, scaled))
    }
}

/// Rescales number with f64 factor
/// returns None if result can't be converted back to original data type
pub fn rescale<T: FromPrimitive + ToPrimitive + Copy>(fact: Percent, a: T, b: T) -> Option<(T, T)> {
    Some((
        T::from_f64((a.to_f64()? * fact).floor())?,
        T::from_f64((b.to_f64()? * fact).floor())?,
    ))
}

/// returns None if fact is 0, -inf, inf or NaN or if win doesn't have a screen
pub fn gtk_win_scale(win: &gtk::Window, rat: Ratio, fact: Percent) -> Option<(i32, i32)> {
    let scr = win.get_screen()?;
    let dims = scr.get_monitor_geometry(scr.get_number());
    let scale_dims = rescale(fact, dims.width, dims.height)?;
    let (_, scaled) = rat.scale(scale_dims.0, scale_dims.1)?;
    Some(scaled)
}

#[test]
fn rat() {
    let (_, rat) = Ratio::new(16, 9).unwrap().scale(1920, 1200).unwrap();
    assert_eq!(rat, (1920, 1080));
}

#[test]
fn ratio_parse() {
    assert!(Ratio::try_from("16x9").is_ok());
    assert!(Ratio::try_from("16x9 ").is_err());
}
