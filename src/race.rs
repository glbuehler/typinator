use crate::render::race::*;

use crossterm::event;
use futures::{FutureExt, StreamExt, select};
use futures_timer::Delay;

use std::time;

pub enum RaceError {
    Aborted,
    Restart,
}

pub async fn run_race(to_type: &[&str]) -> Result<RaceInfo, RaceError> {
    let mut renderer = Renderer::new(to_type);
    renderer.render_full("", time::Duration::ZERO);
    renderer.render_time(time::Duration::ZERO);

    let mut start = None;
    let mut typed = String::new();
    let mut char_iter = crate::char_iter_from_to_type(&to_type);
    let mut next_char = char_iter.next().unwrap();
    let mut mistakes = 0;

    let mut stream = crossterm::event::EventStream::new();
    let mut delay = Delay::new(time::Duration::from_millis(50)).fuse();

    loop {
        let mut next = stream.next().fuse();
        select! {
            _ = delay => {
                renderer.render_time(start.map(|s| time::Instant::now() - s).unwrap_or(time::Duration::ZERO));
                delay = Delay::new(time::Duration::from_millis(50)).fuse();
            },
            e = next => { match e {
                    Some(Ok(event::Event::Resize(w, h))) => {
                        renderer.resize(w.into(), h.into());
                        renderer.render_full(
                            &typed,
                            start.map(|s| time::Instant::now() - s)
                                .unwrap_or(time::Duration::ZERO)
                        );
                    }
                    Some(Ok(event::Event::Key(event::KeyEvent {
                        code: event::KeyCode::Esc,
                        ..
                    }))) => return Err(RaceError::Aborted),
                    Some(Ok(event::Event::Key(event::KeyEvent {
                        code: event::KeyCode::Char('r'),
                        modifiers: event::KeyModifiers::CONTROL,
                        ..
                    }))) => return Err(RaceError::Restart),
                    Some(Ok(event::Event::Key(event::KeyEvent {
                        code: event::KeyCode::Char(c),
                        modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
                        ..
                    }))) => {
                        if start.is_none() {
                            start = Some(time::Instant::now());
                        }
                        renderer.render_char_typed(c);
                        typed.push(c);
                        mistakes += if c == next_char { 0 } else { 1 };
                        match char_iter.next() {
                            Some(c) => next_char = c,
                            None => return Ok(RaceInfo {
                                words: to_type.len(),
                                characters: typed.len(),
                                duration: time::Instant::now() - start.unwrap(),
                                mistakes,
                            }),
                        }
                    },
                    Some(Err(e)) => { dbg!(e); },
                    _ => (),
                };
            },
        }
    }
}

pub struct RaceInfo {
    pub words: usize,
    pub characters: usize,
    pub duration: time::Duration,
    pub mistakes: usize,
}
