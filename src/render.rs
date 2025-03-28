pub mod menu;
pub mod race;

const TEXT_FIELD_RATIO: (f32, f32) = (0.8, 0.5);

const HIDE_CURSOR: &[u8] = b"\x1b[?25l";
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";
const ENABLE_CURSOR_BLINK: &[u8] = b"\x1b[?12l";
const DISABLE_CURSOR_BLINK: &[u8] = b"\x1b[?12h";
const THIN_CURSOR: &[u8] = b"\x1b[5 q";
const RESET_CURSOR: &[u8] = b"\x1b[H";

const CURSOR_DOWN: &[u8] = b"\x1b[B";

const CLEAR_SCREEN: &[u8] = b"\x1b[2J";
const RESET_COLOR: &[u8] = b"\x1b[0m";

fn move_cursor_to_col(col: usize) -> Vec<u8> {
    Vec::from(format!("\x1b[{}G", col + 1))
}

fn move_cursor_to(col: usize, row: usize) -> Vec<u8> {
    Vec::from(format!("\x1b[{};{}H", row + 1, col + 1))
}
