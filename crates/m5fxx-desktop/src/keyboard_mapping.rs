//! Host keyboard mapping for Cardputer simulator.
//!
//! Maps standard PC keyboards (US, German QWERTZ, etc.) to Cardputer physical matrix coordinates.

use egui::Key;
use m5fxx_core::input::KeyCoord;

/// Maps an egui `Key` to a physical Cardputer (row, col) matrix coordinate.
pub fn host_key_to_matrix(key: Key) -> Option<KeyCoord> {
    match key {
        // Row 0: ` 1 2 3 4 5 6 7 8 9 0 - = Del
        Key::Backtick => Some(KeyCoord::new(0, 0)),
        Key::Num1 => Some(KeyCoord::new(0, 1)),
        Key::Num2 => Some(KeyCoord::new(0, 2)),
        Key::Num3 => Some(KeyCoord::new(0, 3)),
        Key::Num4 => Some(KeyCoord::new(0, 4)),
        Key::Num5 => Some(KeyCoord::new(0, 5)),
        Key::Num6 => Some(KeyCoord::new(0, 6)),
        Key::Num7 => Some(KeyCoord::new(0, 7)),
        Key::Num8 => Some(KeyCoord::new(0, 8)),
        Key::Num9 => Some(KeyCoord::new(0, 9)),
        Key::Num0 => Some(KeyCoord::new(0, 10)),
        Key::Minus => Some(KeyCoord::new(0, 11)),
        Key::Equals => Some(KeyCoord::new(0, 12)),
        Key::Backspace => Some(KeyCoord::new(0, 13)),

        // Row 1: Tab Q W E R T Y U I O P [ ] \
        Key::Tab => Some(KeyCoord::new(1, 0)),
        Key::Q => Some(KeyCoord::new(1, 1)),
        Key::W => Some(KeyCoord::new(1, 2)),
        Key::E => Some(KeyCoord::new(1, 3)),
        Key::R => Some(KeyCoord::new(1, 4)),
        Key::T => Some(KeyCoord::new(1, 5)),
        Key::Y => Some(KeyCoord::new(1, 6)),
        Key::U => Some(KeyCoord::new(1, 7)),
        Key::I => Some(KeyCoord::new(1, 8)),
        Key::O => Some(KeyCoord::new(1, 9)),
        Key::P => Some(KeyCoord::new(1, 10)),
        Key::OpenBracket => Some(KeyCoord::new(1, 11)),
        Key::CloseBracket => Some(KeyCoord::new(1, 12)),
        Key::Backslash => Some(KeyCoord::new(1, 13)),

        // Row 2: Fn Shift A S D F G H J K L ; ' Enter
        Key::F1 => Some(KeyCoord::new(2, 0)), // Fn key mapped to F1
        Key::F24 => Some(KeyCoord::new(2, 0)), // Host Fn key (on some keyboards reporting as F24)
        Key::A => Some(KeyCoord::new(2, 2)),
        Key::S => Some(KeyCoord::new(2, 3)),
        Key::D => Some(KeyCoord::new(2, 4)),
        Key::F => Some(KeyCoord::new(2, 5)),
        Key::G => Some(KeyCoord::new(2, 6)),
        Key::H => Some(KeyCoord::new(2, 7)),
        Key::J => Some(KeyCoord::new(2, 8)),
        Key::K => Some(KeyCoord::new(2, 9)),
        Key::L => Some(KeyCoord::new(2, 10)),
        Key::Semicolon => Some(KeyCoord::new(2, 11)),
        Key::Quote => Some(KeyCoord::new(2, 12)),
        Key::Enter => Some(KeyCoord::new(2, 13)),

        // Row 3: Ctrl Opt Alt Z X C V B N M , . / Space
        Key::Z => Some(KeyCoord::new(3, 3)),
        Key::X => Some(KeyCoord::new(3, 4)),
        Key::C => Some(KeyCoord::new(3, 5)),
        Key::V => Some(KeyCoord::new(3, 6)),
        Key::B => Some(KeyCoord::new(3, 7)),
        Key::N => Some(KeyCoord::new(3, 8)),
        Key::M => Some(KeyCoord::new(3, 9)),
        Key::Comma => Some(KeyCoord::new(3, 10)),
        Key::Period => Some(KeyCoord::new(3, 11)),
        Key::Slash => Some(KeyCoord::new(3, 12)),
        Key::Space => Some(KeyCoord::new(3, 13)),

        // Direct host navigation key helpers
        Key::ArrowUp => Some(KeyCoord::new(2, 11)), // ; / Up
        Key::ArrowDown => Some(KeyCoord::new(3, 11)), // . / Down
        Key::ArrowLeft => Some(KeyCoord::new(3, 10)), // , / Left
        Key::ArrowRight => Some(KeyCoord::new(3, 12)), // / / Right
        Key::Escape => Some(KeyCoord::new(0, 0)),   // Esc / `

        _ => None,
    }
}

/// Returns a specific `CardputerKey` override for navigation / control keys
/// so host arrow keys, Enter, Esc, etc. produce the correct event directly
/// without needing Fn to be pressed.
pub fn host_key_to_special(key: Key) -> Option<m5fxx_core::input::CardputerKey> {
    use m5fxx_core::input::CardputerKey;
    match key {
        Key::ArrowUp => Some(CardputerKey::Up),
        Key::ArrowDown => Some(CardputerKey::Down),
        Key::ArrowLeft => Some(CardputerKey::Left),
        Key::ArrowRight => Some(CardputerKey::Right),
        Key::Escape => Some(CardputerKey::Esc),
        Key::Enter => Some(CardputerKey::Enter),
        Key::Backspace => Some(CardputerKey::Backspace),
        Key::Delete => Some(CardputerKey::Delete),
        Key::Tab => Some(CardputerKey::Tab),
        Key::Space => Some(CardputerKey::Space),
        _ => None,
    }
}

/// Maps typed Unicode characters from German or international keyboards into appropriate
/// Cardputer character matrix coordinates if needed.
pub fn char_to_matrix(c: char) -> Option<KeyCoord> {
    let lower = c.to_ascii_lowercase();
    match lower {
        '1'..='9' => Some(KeyCoord::new(0, lower as u8 - b'1' + 1)),
        '0' => Some(KeyCoord::new(0, 10)),
        '-' => Some(KeyCoord::new(0, 11)),
        '=' => Some(KeyCoord::new(0, 12)),
        'q' => Some(KeyCoord::new(1, 1)),
        'w' => Some(KeyCoord::new(1, 2)),
        'e' => Some(KeyCoord::new(1, 3)),
        'r' => Some(KeyCoord::new(1, 4)),
        't' => Some(KeyCoord::new(1, 5)),
        'y' => Some(KeyCoord::new(1, 6)),
        'u' => Some(KeyCoord::new(1, 7)),
        'i' => Some(KeyCoord::new(1, 8)),
        'o' => Some(KeyCoord::new(1, 9)),
        'p' => Some(KeyCoord::new(1, 10)),
        '[' => Some(KeyCoord::new(1, 11)),
        ']' => Some(KeyCoord::new(1, 12)),
        '\\' => Some(KeyCoord::new(1, 13)),
        'a' => Some(KeyCoord::new(2, 2)),
        's' => Some(KeyCoord::new(2, 3)),
        'd' => Some(KeyCoord::new(2, 4)),
        'f' => Some(KeyCoord::new(2, 5)),
        'g' => Some(KeyCoord::new(2, 6)),
        'h' => Some(KeyCoord::new(2, 7)),
        'j' => Some(KeyCoord::new(2, 8)),
        'k' => Some(KeyCoord::new(2, 9)),
        'l' => Some(KeyCoord::new(2, 10)),
        ';' => Some(KeyCoord::new(2, 11)),
        '\'' => Some(KeyCoord::new(2, 12)),
        '\n' => Some(KeyCoord::new(2, 13)),
        'z' => Some(KeyCoord::new(3, 3)),
        'x' => Some(KeyCoord::new(3, 4)),
        'c' => Some(KeyCoord::new(3, 5)),
        'v' => Some(KeyCoord::new(3, 6)),
        'b' => Some(KeyCoord::new(3, 7)),
        'n' => Some(KeyCoord::new(3, 8)),
        'm' => Some(KeyCoord::new(3, 9)),
        ',' => Some(KeyCoord::new(3, 10)),
        '.' => Some(KeyCoord::new(3, 11)),
        '/' => Some(KeyCoord::new(3, 12)),
        ' ' => Some(KeyCoord::new(3, 13)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_key_mapping() {
        assert_eq!(host_key_to_matrix(Key::A), Some(KeyCoord::new(2, 2)));
        assert_eq!(host_key_to_matrix(Key::Space), Some(KeyCoord::new(3, 13)));
        assert_eq!(host_key_to_matrix(Key::ArrowUp), Some(KeyCoord::new(2, 11)));
        assert_eq!(host_key_to_matrix(Key::F1), Some(KeyCoord::new(2, 0))); // Cardputer Fn key
        assert_eq!(char_to_matrix('q'), Some(KeyCoord::new(1, 1)));
        assert_eq!(char_to_matrix('Q'), Some(KeyCoord::new(1, 1)));
    }
}
