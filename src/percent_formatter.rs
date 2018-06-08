use std::fmt;

#[derive(Debug, Clone, Copy)]
enum State {
    GotPercent,
    Normal,
    Skip(usize),
}

pub trait PercentFormatable<W: fmt::Write> {
    fn try_parse(&self, rest: &str, writer: &mut W) -> Result<Option<usize>, fmt::Error>;
}

pub fn percent_format<W, P>(fmt: &str, mut w: &mut W, p: &P) -> Result<(), fmt::Error>
where
    W: fmt::Write,
    P: PercentFormatable<W>,
{
    let mut state = State::Normal;
    let mut last = 0;
    for (i, ch) in fmt.chars().enumerate() {
        state = match state {
            State::Normal => {
                if ch == '%' {
                    write!(w, "{}", &fmt[last..i])?;
                    State::GotPercent
                } else {
                    State::Normal
                }
            }
            State::GotPercent => {
                if ch == '%' {
                    State::Normal
                } else if let Some(skippie) = p.try_parse(&fmt[i..], &mut w)? {
                    last = i + skippie + 1;
                    State::Skip(skippie)
                } else {
                    State::Normal
                }
            }
            State::Skip(0) => State::Normal,
            State::Skip(n) => State::Skip(n - 1),
        }
    }

    if last != fmt.len() {
        write!(w, "{}", &fmt[last..])?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct PercentFormatBuf {
    buf: String,
    format_str: String,
}

impl PercentFormatBuf {
    pub fn new(format: &str) -> Self {
        Self {
            buf: String::new(),
            format_str: format.to_owned(),
        }
    }

    pub fn format<P>(&mut self, p: &P) -> &str
    where
        P: PercentFormatable<String>,
    {
        self.buf.clear();
        percent_format(&self.format_str, &mut self.buf, p).unwrap();
        &self.buf
    }
}
