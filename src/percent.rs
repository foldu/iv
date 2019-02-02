use std::convert::TryFrom;
use std::fmt;
use std::ops;

use noisy_float::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::parse::parse_percent;

/// A percent value. The percentage is never negative or invalid. Subtracting percentages is
/// bottomed out at 0
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Percent(R64);

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.*}%", 2, self.0 * 100.)
    }
}

impl Percent {
    fn step<F: Fn(Self, Self) -> Self>(self, min: Self, rhs: Self, f: F) -> Self {
        let ret = f(self, rhs);
        if rhs.0 == 0. {
            min
        } else {
            match Percent((ret.0 / rhs.0).round() * rhs.0) {
                n if n < min => min,
                n => n,
            }
        }
    }

    pub fn step_next(self, min: Self, inc: Self) -> Self {
        self.step(min, inc, |a, b| a + b)
    }

    pub fn step_prev(self, min: Self, dec: Self) -> Self {
        self.step(min, dec, |a, b| a - b)
    }

    pub fn raw(self) -> R64 {
        self.0
    }
}

macro_rules! derive_from {
    ($t:ty) => {
        impl From<$t> for Percent {
            fn from(n: $t) -> Self {
                Percent(r64(n as f64 / 100.))
            }
        }
    };
}

#[derive(Fail, Debug, Clone)]
pub enum PercentError {
    #[fail(display = "Can't create negative percentages: {} is negative", _0)]
    IsNegative(i64),
    #[fail(display = "Can't create percentage from this float: {}", _0)]
    InvalidFloat(f64),
    #[fail(display = "Can't parse percentage from \"{}\"", _0)]
    Parse(String),
}

derive_from!(u8);
derive_from!(u16);
derive_from!(u32);
derive_from!(u64);
derive_from!(usize);

macro_rules! derive_float_into {
    ($t:ty) => {
        impl From<Percent> for $t {
            fn from(p: Percent) -> Self {
                (p.0 / 100.)
            }
        }
    };
}

derive_float_into!(R64);

macro_rules! derive_int_try_from {
    ($t:ty) => {
        impl TryFrom<$t> for Percent {
            type Error = PercentError;
            fn try_from(n: $t) -> Result<Percent, Self::Error> {
                if n < 0 {
                    Err(PercentError::IsNegative(n as i64))
                } else {
                    Ok(Percent::from(n as u64))
                }
            }
        }
    };
}

derive_int_try_from!(i8);
derive_int_try_from!(i16);
derive_int_try_from!(i32);
derive_int_try_from!(i64);

macro_rules! derive_float_try_from {
    ($t:ty) => {
        impl TryFrom<$t> for Percent {
            type Error = PercentError;
            fn try_from(n: $t) -> Result<Percent, Self::Error> {
                if n.is_finite() && n >= 0. {
                    Ok(Percent(r64(n as f64)))
                } else {
                    Err(PercentError::InvalidFloat(n as f64))
                }
            }
        }
    };
}

derive_float_try_from!(f32);
derive_float_try_from!(f64);

impl<'a> TryFrom<&'a str> for Percent {
    type Error = PercentError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        parse_percent(s)
            .and_then(|n| Percent::try_from(n / 100.).ok())
            .ok_or_else(|| PercentError::Parse(s.to_owned()))
    }
}

macro_rules! derive_mul {
    ($t:ty) => {
        impl ops::Mul<$t> for Percent {
            type Output = $t;
            fn mul(self, rhs: $t) -> Self::Output {
                (r64(rhs as f64) * self.0).raw() as $t
            }
        }

        impl ops::Mul<Percent> for $t {
            type Output = $t;
            fn mul(self, rhs: Percent) -> Self::Output {
                (r64(self as f64) * rhs.0).raw() as $t
            }
        }
    };
}

derive_mul!(u8);
derive_mul!(u16);
derive_mul!(u32);
derive_mul!(u64);
derive_mul!(usize);

derive_mul!(i8);
derive_mul!(i16);
derive_mul!(i32);
derive_mul!(i64);

derive_mul!(f32);
derive_mul!(f64);

impl ops::Add for Percent {
    type Output = Percent;
    fn add(self, rhs: Percent) -> Self::Output {
        Percent(self.0 + rhs.0)
    }
}

impl ops::Sub for Percent {
    type Output = Percent;
    fn sub(self, rhs: Percent) -> Self::Output {
        match self.0 - rhs.0 {
            n if n < 0. => Percent(r64(0.)),
            n => Percent(n),
        }
    }
}

impl Serialize for Percent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for Percent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PercentVisitor;
        impl<'de> de::Visitor<'de> for PercentVisitor {
            type Value = Percent;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a percentage")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Percent::try_from(value).map_err(|e| E::custom(format!("{}", e)))
            }
        }

        deserializer.deserialize_str(PercentVisitor)
    }
}

impl Default for Percent {
    fn default() -> Self {
        Percent::from(0_u32)
    }
}

#[test]
fn percent_from() {
    assert_eq!(Percent::from(10_u32) * 100_u32, 10);
    assert_eq!(100_u32 * Percent::from(10_u32), 10);
}

#[test]
fn percent_try_from() {
    use std::f64;
    assert!(Percent::try_from(1).is_ok());
    assert!(Percent::try_from(0.5).is_ok());
    assert!(Percent::try_from(-1).is_err());
    assert!(Percent::try_from(f64::NAN).is_err());
}

#[test]
fn percent_math() {
    assert_eq!(
        Percent::from(50_u32) + Percent::from(50_u32),
        Percent::from(100_u32)
    );

    assert_eq!(
        Percent::from(0_u32) - Percent::from(20_u32),
        Percent::from(0_u32)
    );
}

#[test]
fn percent_step() {
    assert_eq!(
        Percent::from(50_u32).step_next(Percent::from(5_u32), Percent::from(25_u32)),
        Percent::from(75_u32)
    );
    assert_eq!(
        Percent::from(28_u32).step_prev(Percent::from(25_u32), Percent::from(25_u32)),
        Percent::from(25_u32)
    );
}

#[test]
fn parse_percent_logic() {
    assert!(Percent::try_from("-20%").is_err())
}
