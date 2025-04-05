use std::io::{self, Write};

use crossterm::terminal;

use super::*;
use crate::race;

pub fn render_menu(race_result: race::RaceInfo) {
    let time = format!("{:.2}s", race_result.duration.as_secs_f32());
    let wpm = if !race_result.duration.is_zero() {
        format!(
            "{:.2}",
            (race_result.words as f32 / (race_result.duration.as_secs_f32() / 60.0))
        )
    } else {
        "N/A".to_string()
    };
    let accuracy = if race_result.characters != 0 {
        format!(
            "{:.2}%",
            (100.0 * (1.0 - (race_result.mistakes as f32 / race_result.characters as f32)))
        )
    } else {
        "N/A".to_string()
    };

    render_results(&[
        ("Words typed", &race_result.words.to_string()),
        ("Letters typed", &race_result.characters.to_string()),
        ("Mistakes", &race_result.mistakes.to_string()),
        ("Time", &time),
        ("WPM", &wpm),
        ("Accuracy", &accuracy),
    ])
}

fn render_results(res: &[(&str, &str)]) {
    let (w, h) = terminal::size().unwrap();
    let (w, h) = (w as usize, h as usize);
    let mut row = (h - res.len()) / 2;

    let mut buf = vec![];
    buf.extend(CLEAR_SCREEN);
    buf.extend(HIDE_CURSOR);

    for (l, r) in res {
        let col = w / 2 - l.chars().count() - 1;
        buf.extend(move_cursor_to(col, row));
        buf.extend(l.as_bytes());
        buf.push(b' ');
        buf.extend(r.as_bytes());

        row += 1;
    }

    io::stdout().write_all(&buf).unwrap();
    io::stdout().flush().unwrap();
}
