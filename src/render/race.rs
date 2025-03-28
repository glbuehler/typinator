use std::io::{self, Write};

use crossterm::terminal;

use super::*;

const TO_TYPE: &[u8] = b"\x1b[3;37m";
const CORRECT: &[u8] = b"\x1b[1;97m";
const MISTAKE: &[u8] = b"\x1b[37;48;5;52m";

pub struct Renderer<'a, 'b: 'a> {
    to_type: &'a [&'b str],
    cursor: (usize, usize),
    line_lens: Vec<usize>,
    line_words: Vec<usize>,

    top_left: (usize, usize),
    size: (usize, usize),
}

impl<'a, 'b: 'a> Renderer<'a, 'b> {
    pub fn new(to_type: &'a [&'b str]) -> Self {
        let mut s = Self {
            to_type,
            cursor: (0, 0),
            line_lens: vec![],
            line_words: vec![],

            top_left: (0, 0),
            size: (0, 0),
        };
        let Ok((w, h)) = terminal::size() else {
            panic!("cannot get terminal size");
        };
        s.resize(w.into(), h.into());
        s
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let mut typed_len = self
            .line_lens
            .iter()
            .take(self.cursor.1)
            .map(|l| l + 1) // one space at end
            .sum::<usize>()
            + self.cursor.0;

        self.size = (
            (w as f32 * TEXT_FIELD_RATIO.0) as usize,
            (h as f32 * TEXT_FIELD_RATIO.1) as usize,
        );
        self.top_left = (w / 2 - self.size.0 / 2, h / 2 - self.size.1 / 2);

        let mut iter = self.to_type.iter();
        self.line_lens.clear();
        self.line_lens
            .push(iter.next().map(|s| s.chars().count()).unwrap_or(0));

        self.line_words.clear();
        self.line_words.push(1);

        for w in iter {
            let l = w.chars().count();
            let len = self.line_lens.last_mut().unwrap();
            if *len + l + 1 >= self.size.0 {
                self.line_lens.push(l);
                self.line_words.push(1);
            } else {
                *len += l + 1;
                *self.line_words.last_mut().unwrap() += 1;
            }
        }

        self.cursor = (0, 0);
        for l in self.line_lens.iter() {
            if typed_len < *l {
                break;
            }
            typed_len -= l + 1; // one space at end
            self.cursor.1 += 1;
        }
        self.cursor.0 = typed_len;
    }

    pub fn render_full(&self, typed: &str) {
        let mut buf = vec![];
        buf.extend(CLEAR_SCREEN);
        buf.extend(move_cursor_to(self.top_left.0, self.top_left.1));
        buf.extend(SHOW_CURSOR);
        buf.extend(DISABLE_CURSOR_BLINK);
        buf.extend(THIN_CURSOR);
        buf.extend(TO_TYPE);

        let mut i = 0;
        let mut typed_iter = typed.chars();
        for l in self.line_words.iter() {
            for c in crate::char_iter_from_to_type(&self.to_type[i..i + *l]) {
                buf.extend(if let Some(t) = typed_iter.next() {
                    if t == c { CORRECT } else { MISTAKE }
                } else {
                    TO_TYPE
                });
                buf.extend(c.to_string().as_bytes());
                buf.extend(RESET_COLOR);
            }
            typed_iter.next(); // space at end of line
            i += l;

            buf.extend(move_cursor_to_col(self.top_left.0));
            buf.extend(CURSOR_DOWN);
        }
        buf.extend(move_cursor_to(
            self.top_left.0 + self.cursor.0,
            self.top_left.1 + self.cursor.1,
        ));

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();
    }

    pub fn render_char_typed(&mut self, c: char) {
        let mut buf = vec![];

        if self.cursor.0 + 1 > *self.line_lens.get(self.cursor.1).unwrap_or(&0) {
            self.cursor.0 = 0;
            self.cursor.1 += 1;
        } else {
            let Some(ch) = self.get_char_under_cursor() else {
                panic!("cursor not over char");
            };
            if c == ch {
                buf.extend(CORRECT);
            } else {
                buf.extend(MISTAKE);
            }
            buf.extend(ch.to_string().as_bytes());
            buf.extend(RESET_COLOR);
            self.cursor.0 += 1;
        }

        buf.extend(move_cursor_to(
            self.top_left.0 + self.cursor.0,
            self.top_left.1 + self.cursor.1,
        ));

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();
    }

    fn get_char_under_cursor(&self) -> Option<char> {
        let mut iter = self.to_type.iter().cloned();
        for w in self.line_words.iter().take(self.cursor.1) {
            for _ in 0..*w {
                iter.next();
            }
        }
        iter.take(*self.line_words.get(self.cursor.1).unwrap_or(&0))
            .intersperse(" ")
            .map(|s| s.chars())
            .flatten()
            .nth(self.cursor.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
