use std::{
    io::{self, Write},
    time,
};

use crossterm::terminal;

use super::*;

const TO_TYPE: &[u8] = b"\x1b[3;37m";
const CORRECT: &[u8] = b"\x1b[1;97m";
const MISTAKE: &[u8] = b"\x1b[1;97;48;2;96;16;16m"; //  b"\x1b[37;48;5;88m";
const TO_TYPE_SUBTLE: &[u8] = b"\x1b[3;38;5;240m";
const CORRECT_SUBTLE: &[u8] = b"\x1b[1;38;5;242m";
const MISTAKE_SUBTLE: &[u8] = b"\x1b[1;38;5;242;48;2;64;0;0m";

#[derive(Debug)]
pub struct Renderer<'a, 'b: 'a> {
    to_type: &'a [&'b str],
    cursor: (usize, usize),
    scroll: usize,
    line_lens: Vec<usize>,
    line_words: Vec<usize>,

    size: (usize, usize),
    text_field_top_left: (usize, usize),
    text_field_size: (usize, usize),
}

impl<'a, 'b: 'a> Renderer<'a, 'b> {
    pub fn new(to_type: &'a [&'b str]) -> Self {
        let mut s = Self {
            to_type,
            cursor: (0, 0),
            scroll: 0,
            line_lens: vec![],
            line_words: vec![],

            size: (0, 0),
            text_field_top_left: (0, 0),
            text_field_size: (0, 0),
        };
        let Ok((w, h)) = terminal::size() else {
            panic!("cannot get terminal size");
        };
        s.resize(w.into(), h.into());
        s
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.size = (w, h);
        let mut typed_len = self
            .line_lens
            .iter()
            .take(self.scroll + self.cursor.1)
            .map(|l| l + 1) // one space at end
            .sum::<usize>()
            + self.cursor.0;

        self.text_field_size = (
            (w as f32 * TEXT_FIELD_RATIO.0) as usize,
            (h as f32 * TEXT_FIELD_RATIO.1) as usize,
        );
        self.text_field_top_left = (
            w / 2 - self.text_field_size.0 / 2,
            h / 2 - self.text_field_size.1 / 2,
        );

        let mut iter = self.to_type.iter();
        self.line_lens.clear();
        self.line_lens
            .push(iter.next().map(|s| s.chars().count()).unwrap_or(0));

        self.line_words.clear();
        self.line_words.push(1);

        for w in iter {
            let l = w.chars().count();
            let len = self.line_lens.last_mut().unwrap();
            if *len + l + 1 >= self.text_field_size.0 {
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
            typed_len = typed_len.checked_sub(l + 1).unwrap_or(0); // one space at end
            self.cursor.1 += 1;
        }
        self.cursor.0 = typed_len;
    }

    pub fn render_full(&self, typed: &str, time: time::Duration) {
        let mut buf = vec![];
        buf.extend(CLEAR_SCREEN);
        buf.extend(SHOW_CURSOR);
        buf.extend(DISABLE_CURSOR_BLINK);
        buf.extend(THIN_CURSOR);

        buf.extend(self.text_field_buf(typed));
        buf.extend(self.time_buf(time));

        buf.extend(move_cursor_to(
            self.text_field_top_left.0 + self.cursor.0,
            self.text_field_top_left.1 + self.cursor.1,
        ));

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();
    }

    pub fn render_char_typed(&mut self, c: char, typed: &str) {
        let mut buf = vec![];

        if self.cursor.0 + 1
            > *self
                .line_lens
                .get(self.scroll + self.cursor.1)
                .unwrap_or(&0)
        {
            if c != ' ' {
                buf.extend(MISTAKE);
                buf.push(b' ');
                buf.extend(RESET_COLOR);
            }
            self.cursor.0 = 0;

            if self.cursor.1 >= self.text_field_size.1 / 2
                && self.scroll + self.text_field_size.1 < self.line_words.len()
            {
                self.scroll += 1;
                buf.extend(self.text_field_buf(typed))
            } else {
                self.cursor.1 += 1;
            }
        } else {
            let Some(ch) = self.get_char_under_cursor() else {
                std::thread::sleep(time::Duration::from_secs(5));
                panic!("cursor not over char {:?}", self);
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
            self.text_field_top_left.0 + self.cursor.0,
            self.text_field_top_left.1 + self.cursor.1,
        ));

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();
    }

    pub fn render_time(&self, time: time::Duration) {
        let mut buf = self.time_buf(time);

        buf.extend(move_cursor_to(
            self.text_field_top_left.0 + self.cursor.0,
            self.text_field_top_left.1 + self.cursor.1,
        ));

        io::stdout().write_all(&buf).unwrap();
        io::stdout().flush().unwrap();
    }

    fn time_buf(&self, time: time::Duration) -> Vec<u8> {
        let mut buf = vec![];
        let time_str = format!("{:.2}", time.as_secs_f32());
        let l = time_str.chars().count();

        buf.extend(move_cursor_to((self.size.0 - l) / 2, 1));
        buf.extend(time_str.as_bytes());
        buf
    }

    fn text_field_buf(&self, typed: &str) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(move_cursor_to(
            self.text_field_top_left.0,
            self.text_field_top_left.1,
        ));

        let at_bottom = self.scroll + self.text_field_size.1 >= self.line_words.len();

        let mut len_iter = self.line_lens.iter();
        let mut to_type_iter = crate::char_iter_from_to_type(self.to_type);
        let mut typed_iter = typed.chars();

        for _ in 0..self.scroll {
            let l = len_iter.next().expect("scroll greater than line number");
            for _ in 0..*l + 1 {
                to_type_iter.next();
                typed_iter.next();
            }
        }

        if self.scroll > 0 {
            let l = len_iter.next().expect("scroll not 0 but no lines");
            for _ in 0..*l + 1 {
                let Some(tt) = to_type_iter.next() else { break };
                buf.extend(if let Some(t) = typed_iter.next() {
                    if t == tt {
                        CORRECT_SUBTLE
                    } else {
                        MISTAKE_SUBTLE
                    }
                } else {
                    TO_TYPE_SUBTLE
                });
                buf.extend(tt.to_string().as_bytes());
                buf.extend(RESET_COLOR);
            }
            buf.extend(vec![b' '; self.text_field_size.0 - l]);

            buf.extend(move_cursor_to_col(self.text_field_top_left.0));
            buf.extend(CURSOR_DOWN);
        }

        let mut take = self.text_field_size.1;
        if self.scroll > 0 {
            take -= 1;
        }
        if !at_bottom {
            take -= 1;
        }

        while take > 0 {
            let Some(l) = len_iter.next() else { break };
            for _ in 0..*l + 1 {
                let Some(tt) = to_type_iter.next() else { break };
                buf.extend(if let Some(t) = typed_iter.next() {
                    if t == tt { CORRECT } else { MISTAKE }
                } else {
                    TO_TYPE
                });
                buf.extend(tt.to_string().as_bytes());
                buf.extend(RESET_COLOR);
            }
            buf.extend(vec![b' '; self.text_field_size.0 - l]);

            buf.extend(move_cursor_to_col(self.text_field_top_left.0));
            buf.extend(CURSOR_DOWN);
            take -= 1;
        }

        if !at_bottom {
            let l = len_iter
                .next()
                .expect("not at bottom but no line left???!!!");
            for _ in 0..*l + 1 {
                let Some(tt) = to_type_iter.next() else { break };
                buf.extend(if let Some(t) = typed_iter.next() {
                    if t == tt {
                        CORRECT_SUBTLE
                    } else {
                        MISTAKE_SUBTLE
                    }
                } else {
                    TO_TYPE_SUBTLE
                });
                buf.extend(tt.to_string().as_bytes());
                buf.extend(RESET_COLOR);
            }
            buf.extend(vec![b' '; self.text_field_size.0 - l]);
        }

        buf
    }

    fn get_char_under_cursor(&self) -> Option<char> {
        let mut iter = self.to_type.iter().cloned();
        for w in self.line_words.iter().take(self.scroll + self.cursor.1) {
            for _ in 0..*w {
                iter.next();
            }
        }
        iter.take(
            *self
                .line_words
                .get(self.scroll + self.cursor.1)
                .unwrap_or(&0),
        )
        .intersperse(" ")
        .map(|s| s.chars())
        .flatten()
        .nth(self.cursor.0)
    }
}
