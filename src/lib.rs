#![feature(iter_intersperse)]

use std::{self, io, time};

use crossterm::{event, execute, terminal};
use futures::{StreamExt, future::FutureExt, select};
use lazy_static::lazy_static;
use rand;

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
    execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .unwrap();
    terminal::disable_raw_mode().unwrap();
}

fn char_iter_from_to_type(to_type: &[&str]) -> impl Iterator<Item = char> {
    to_type
        .iter()
        .intersperse_with(|| &" ")
        .map(|s| s.chars())
        .flatten()
}

pub async fn run() {
    let mut to_type = vec![];
    let mut reader = event::EventStream::new();

    'outer: loop {
        to_type.clear();
        for _ in 0..30 {
            let rnd = rand::random_range(0..WORDS.len());
            to_type.push(WORDS[rnd]);
        }

        let info = match race::run_race(&to_type).await {
            Ok(info) => info,
            Err(race::RaceError::Aborted) => break,
            Err(race::RaceError::Restart) => continue,
        };

        futures_timer::Delay::new(time::Duration::from_millis(500)).await;
        render::menu::render_menu(info);

        loop {
            let mut next = reader.next().fuse();
            select! {
                e = next => { match e {
                        Some(Ok(event::Event::Key(event::KeyEvent {
                            code: event::KeyCode::Esc,
                            ..
                        }))) => break 'outer,
                        Some(Ok(event::Event::Key(event::KeyEvent {
                            code: event::KeyCode::Enter,
                            ..
                        }))) => break,
                        _ => (),
                    }
                }
            }
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
