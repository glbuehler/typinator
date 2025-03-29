#![feature(iter_intersperse)]

use std::{self, io, sync};

use crossterm::{event, execute, terminal};
use lazy_static::lazy_static;
use rand;

mod input;
mod race;
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

fn char_iter_from_to_type(to_type: &[&str]) -> impl Iterator<Item = char> {
    to_type
        .iter()
        .intersperse_with(|| &" ")
        .map(|s| s.chars())
        .flatten()
}

pub fn run() {
    let mut to_type = vec![];

    let (input_chan, run) = input::spawn_input_thread();

    'outer: loop {
        to_type.clear();
        for _ in 0..30 {
            let random = rand::random_range(0..WORDS.len());
            to_type.push(WORDS[random]);
        }

        let result = match race::run_race(&to_type, &input_chan) {
            Ok(r) => r,
            Err(race::RaceError::Aborted) => break 'outer,
            Err(race::RaceError::Restart) => continue,
        };
        std::thread::sleep(std::time::Duration::from_millis(500));

        render::menu::render_menu(result);

        for e in input_chan.iter() {
            match e {
                event::Event::Key(event::KeyEvent {
                    code: event::KeyCode::Enter,
                    ..
                }) => break,
                event::Event::Key(event::KeyEvent {
                    code: event::KeyCode::Esc,
                    ..
                }) => break 'outer,
                _ => (),
            }
        }
    }
    run.store(false, sync::atomic::Ordering::Relaxed);
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
