#![allow(unused)]
#![feature(iter_intersperse)]

use std::{self, io, time};

use crossterm::{event, execute, terminal};
use lazy_static::lazy_static;
use rand;

mod input;
mod render;

static WORD_FILE: &str = include_str!("../words.txt");

lazy_static! {
    static ref WORDS: Vec<&'static str> = Vec::from_iter(WORD_FILE.lines());
}

pub fn enter() {
    terminal::enable_raw_mode().unwrap();
    execute!(io::stdout(), terminal::EnterAlternateScreen).unwrap();
}

pub fn exit() {
    execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
}

pub fn char_iter_from_to_type(to_type: &[&str]) -> impl Iterator<Item = char> {
    to_type
        .iter()
        .intersperse_with(|| &" ")
        .map(|s| s.chars())
        .flatten()
}

pub fn run() {
    let mut to_type = vec![];

    while to_type.len() < 50 {
        let random = rand::random_range(0..WORDS.len());
        to_type.push(WORDS[random]);
    }

    let mut renderer = render::Renderer::new(&to_type);
    renderer.render_full("");

    let input_chan = input::spawn_input_thread();

    let mut iter = input_chan.iter();
    let Some(e) = iter.next() else {
        return;
    };
    let mut typed = String::new();
    let mut start = time::Instant::now();

    for event in std::iter::once(e).chain(iter) {
        match event {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            }) => break,
            event::Event::Resize(w, h) => {
                renderer.resize(w.into(), h.into());
                renderer.render_full(&typed);
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char(c),
                ..
            }) => {
                typed.push(c);
                renderer.render_char_typed(c);
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iter() {
        let to_type = &["hello", "world", "!"];
        let mut iter = char_iter_from_to_type(to_type);

        assert_eq!(iter.next(), Some('h'));
        assert_eq!(iter.next(), Some('e'));
        assert_eq!(iter.next(), Some('l'));
        assert_eq!(iter.next(), Some('l'));
        assert_eq!(iter.next(), Some('o'));
        assert_eq!(iter.next(), Some(' '));
        assert_eq!(iter.next(), Some('w'));
        assert_eq!(iter.next(), Some('o'));
        assert_eq!(iter.next(), Some('r'));
        assert_eq!(iter.next(), Some('l'));
        assert_eq!(iter.next(), Some('d'));
        assert_eq!(iter.next(), Some(' '));
        assert_eq!(iter.next(), Some('!'));
    }
}
