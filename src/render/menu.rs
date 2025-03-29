use std::io::{self, Write};

use super::*;
use crate::race;

pub fn render_menu(race_result: race::RaceInfo) {
    let mut buf = vec![];
    buf.extend(CLEAR_SCREEN);
    buf.extend(RESET_CURSOR);
    buf.extend(HIDE_CURSOR);

    buf.extend(
        format!(
            "Words typed:\t{}\r\nTime:\t\t{:.2}\r\nMistakes:\t{}\r\nWPM:\t\t{:.2}\r\nAccuracy:\t{:.2}%",
            race_result.words,
            race_result.duration.as_secs_f32(),
            race_result.mistakes,
            race_result.words as f32 / (race_result.duration.as_secs_f32() / 60.0),
            100.0 * (1.0 - (race_result.mistakes as f32 / race_result.characters as f32)),
        )
        .as_bytes(),
    );

    io::stdout().write_all(&buf).unwrap();
    io::stdout().flush().unwrap();
}
