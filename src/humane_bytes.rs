use std::convert::{From, TryFrom};
use std::fmt;

use failure;
use num;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use parse::parse_human_readable_bytes;

#[derive(Debug, Clone, Copy)]
pub struct HumaneBytes(u64);

impl From<usize> for HumaneBytes {
    fn from(other: usize) -> Self {
        Self { 0: other as u64 }
    }
}

impl From<HumaneBytes> for usize {
    fn from(other: HumaneBytes) -> Self {
        other.0 as usize
    }
}

impl From<u64> for HumaneBytes {
    fn from(other: u64) -> Self {
        Self { 0: other }
    }
}

impl From<HumaneBytes> for u64 {
    fn from(other: HumaneBytes) -> Self {
        other.0
    }
}

impl<'a> TryFrom<&'a str> for HumaneBytes {
    type Error = failure::Error;
    fn try_from(s: &str) -> Result<Self, failure::Error> {
        parse_human_readable_bytes(s).map(HumaneBytes)
    }
}

impl fmt::Display for HumaneBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 < 1000 {
            write!(f, "{}B", self.0)
        } else {
            let suff_tbl = ['k', 'M', 'G', 'T'];
            let lg10 = f64::log10(self.0 as f64);
            let thing = f64::floor(num::clamp(lg10, 3., 12.) / 3.) - 1.;
            assert!(thing >= 0.);
            let i = thing as usize;
            write!(
                f,
                "{:.2}{}B",
                self.0 as f64 / f64::powi(10., ((i + 1) * 3) as i32),
                suff_tbl[i]
            )
        }
    }
}

impl<'de> Deserialize<'de> for HumaneBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct HumaneBytesVisitor;

        impl<'de> Visitor<'de> for HumaneBytesVisitor {
            type Value = HumaneBytes;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a human readable si number of bytes")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                HumaneBytes::try_from(value)
                    .map_err(|e| E::custom(&format!("Can't parse as readable si bytes: {}", e)))
            }
        }

        deserializer.deserialize_str(HumaneBytesVisitor)
    }
}

impl Serialize for HumaneBytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[test]
fn humane_bytes_format() {
    assert_eq!(&format!("{}", HumaneBytes(139)), "139B");
    assert_eq!(&format!("{}", HumaneBytes(1390)), "1.39kB");
    assert_eq!(
        &format!("{}", HumaneBytes(1_390_000_000_000_000_000)),
        "1390000.00TB"
    );
}
