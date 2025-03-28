use std::{
    io::{self, Write},
    time,
};

use crossterm::terminal;

use crate::WORDS;

const TEXT_FIELD_RATIO: (f32, f32) = (0.8, 0.5);

const THIN_CURSOR: &[u8] = b"\x1b[5 q";
const RESET_CURSOR: &[u8] = b"\x1b[H";
const CURSOR_DOWN: &[u8] = b"\x1b[B";
const CLEAR_SCREEN: &[u8] = b"\x1b[2J";

const TO_TYPE: &[u8] = b"\x1b[3;37m";
const CORRECT: &[u8] = b"\x1b[1;97m";
const MISTAKE: &[u8] = b"\x1b[37;48;5;52m";
const RESET_COLOR: &[u8] = b"\x1b[0m";

fn move_cursor_to_col(col: usize) -> Vec<u8> {
    Vec::from(format!("\x1b[{}G", col + 1))
}

fn move_cursor_to(col: usize, row: usize) -> Vec<u8> {
    Vec::from(format!("\x1b[{};{}H", row + 1, col + 1))
}

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
    }

    pub fn render_full(&self, typed: &str) {
        let mut buf = vec![];
        buf.extend(CLEAR_SCREEN);
        buf.extend(move_cursor_to(self.top_left.0, self.top_left.1));
        buf.extend(THIN_CURSOR);
        buf.extend(TO_TYPE);

        let mut iter = self.to_type.iter();
        for l in self.line_words.iter() {
            for _ in 0..*l {
                let Some(next) = iter.next() else {
                    panic!("invalid line word length");
                };
                buf.extend(next.as_bytes());
                buf.push(b' ');
            }
            buf.pop();
            buf.extend(move_cursor_to_col(self.top_left.0));
            buf.extend(CURSOR_DOWN);
        }
        buf.extend(RESET_COLOR);
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
