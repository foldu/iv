use std::convert::{From, TryFrom};
use std::fmt;

use failure;
use num;

use parse::parse_human_readable_bytes;

#[derive(Debug, Clone, Copy)]
pub struct HumaneBytes(usize);

impl From<usize> for HumaneBytes {
    fn from(other: usize) -> Self {
        Self { 0: other }
    }
}

impl From<HumaneBytes> for usize {
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

#[test]
fn humane_bytes_format() {
    assert_eq!(&format!("{}", HumaneBytes(139)), "139B");
    assert_eq!(&format!("{}", HumaneBytes(1390)), "1.39kB");
    assert_eq!(
        &format!("{}", HumaneBytes(1_390_000_000_000_000_000)),
        "1390000.00TB"
    );
}
