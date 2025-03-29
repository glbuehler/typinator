use crate::render::race::*;

use crossterm::event;

use std::{sync::mpsc, time};

pub enum RaceError {
    Aborted,
    Restart,
}

pub fn run_race(
    to_type: &[&str],
    input_chan: &mpsc::Receiver<event::Event>,
) -> Result<RaceInfo, RaceError> {
    let mut renderer = Renderer::new(&to_type);
    renderer.render_full("");

    let mut start = None;
    let mut typed = String::new();
    let mut char_iter = crate::char_iter_from_to_type(&to_type);
    let mut mistakes = 0;
    let mut next_char = char_iter.next().unwrap();

    for event in input_chan.iter() {
        match event {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            }) => return Err(RaceError::Aborted),
            event::Event::Resize(w, h) => {
                renderer.resize(w.into(), h.into());
                renderer.render_full(&typed);
            }
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('r'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            }) => return Err(RaceError::Restart),
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
                ..
            }) => {
                if start.is_none() {
                    start = Some(time::Instant::now());
                }
                typed.push(c);
                renderer.render_char_typed(c);
                mistakes += if c != next_char { 1 } else { 0 };
                match char_iter.next() {
                    Some(ch) => next_char = ch,
                    None => {
                        return Ok(RaceInfo {
                            words: to_type.len(),
                            characters: typed.len(),
                            duration: time::Instant::now() - start.unwrap(),
                            mistakes,
                        });
                    }
                }
            }
            _ => (),
        }
    }
    unreachable!()
}

pub struct RaceInfo {
    pub words: usize,
    pub characters: usize,
    pub duration: time::Duration,
    pub mistakes: usize,
}
