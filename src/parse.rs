use failure;
use nom::{recognize_float, types::CompleteStr};

named!(
    p_f64(CompleteStr) -> f64,
    flat_map!(call!(recognize_float), parse_to!(f64))
);

named!(
    human_bytes(CompleteStr) -> usize,
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
            (ret * multiplier as f64) as usize
        }).unwrap_or(ret as usize))
    )
);

pub fn parse_human_readable_bytes<'a>(s: &'a str) -> Result<usize, failure::Error> {
    let (_, ret) = human_bytes(CompleteStr(&s))
        .map_err(|e| format_err!("Can't parse as human readable bytes: {}", e))?;
    Ok(ret)
}

#[test]
fn parse_bytes() {
    assert!(parse_human_readable_bytes("test").is_err());
    assert!(parse_human_readable_bytes("1000B").is_ok());
    assert!(parse_human_readable_bytes("1000b").is_ok());
    assert!(parse_human_readable_bytes("1000kb").is_ok());
    assert!(parse_human_readable_bytes("1000tB").is_ok());
}
