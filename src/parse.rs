use failure;
use nom::{recognize_float, types::CompleteStr};

named!(
    p_f64(CompleteStr) -> f64,
    flat_map!(call!(recognize_float), parse_to!(f64))
);

named!(
    human_bytes(CompleteStr) -> u64,
    do_parse!(
        ret: p_f64 >>
        prefix: opt!(one_of!("kmgtKMGT")) >>
        one_of!("Bb") >>
        eof!() >>
        (prefix.map(|p| {
            let multiplier = match p {
                'k' | 'K' => 1_000,
                'm' | 'M' => 1_000_000,
                'g' | 'G' => 1_000_000_000,
                't' | 'T' => 1_000_000_000_000_u64,
                _ => unreachable!(),
            };
            (ret * multiplier as f64) as u64
        }).unwrap_or(ret as u64))
    )
);

pub fn parse_human_readable_bytes<'a>(s: &'a str) -> Result<u64, failure::Error> {
    let (_, ret) = human_bytes(CompleteStr(&s))
        .map_err(|e| format_err!("Can't parse as human readable bytes: {}", e))?;
    Ok(ret)
}

named!(
    percent(CompleteStr) -> f64,
    do_parse!(
        ret: p_f64 >>
        char!('%') >>
        eof!() >>
        (ret)
    )
);

pub fn parse_percent(s: &str) -> Option<f64> {
    percent(CompleteStr(&s)).ok().map(|(_, ret)| ret)
}

#[test]
fn parse_bytes() {
    assert!(parse_human_readable_bytes("test").is_err());
    assert!(parse_human_readable_bytes("1000B").is_ok());
    assert!(parse_human_readable_bytes("1000b").is_ok());
    assert!(parse_human_readable_bytes("1000kb").is_ok());
    assert!(parse_human_readable_bytes("1000tB").is_ok());
}

#[test]
fn parse_percent_f64() {
    assert!(parse_percent("20%").is_some());
    assert!(parse_percent("20").is_none());
    assert!(parse_percent("20% ").is_none());
    assert!(parse_percent("test").is_none());
    assert!(parse_percent("%").is_none());
    assert!(parse_percent("").is_none());
}
