pub struct PercentFormatter {
    buf: String,
    format_str: String,
}

#[derive(Debug, Clone, Copy)]
enum State {
    GotPercent,
    Normal,
    Skip(usize),
}

pub trait PercentFormatable {
    fn try_parse(&self, rest: &str, buf: &mut String) -> Option<usize>;
}

impl PercentFormatter {
    pub fn new(format: &str) -> Self {
        Self {
            buf: String::new(),
            format_str: format.to_owned(),
        }
    }

    pub fn format<P: PercentFormatable>(&mut self, p: &P) -> &str {
        self.buf.clear();
        let mut state = State::Normal;
        let mut last = 0;
        for (i, ch) in self.format_str.chars().enumerate() {
            state = match state {
                State::Normal => {
                    if ch == '%' {
                        self.buf.push_str(&self.format_str[last..i]);
                        State::GotPercent
                    } else {
                        State::Normal
                    }
                }
                State::GotPercent => {
                    if ch == '%' {
                        State::Normal
                    } else {
                        if let Some(skippie) = p.try_parse(&self.format_str[i..], &mut self.buf) {
                            last = i + skippie + 1;
                            State::Skip(skippie)
                        } else {
                            State::Normal
                        }
                    }
                }
                State::Skip(0) => State::Normal,
                State::Skip(n) => State::Skip(n - 1),
            }
        }

        if last != self.format_str.len() {
            self.buf.push_str(&self.format_str[last..]);
        }

        &self.buf
    }
}
