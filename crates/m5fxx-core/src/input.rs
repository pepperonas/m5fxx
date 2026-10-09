//! Cardputer keyboard matrix, modifier tracking, and input state.

use std::collections::HashSet;

pub const ROWS: usize = 4;
pub const COLS: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCoord {
    pub row: u8,
    pub col: u8,
}

impl KeyCoord {
    pub const fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyModifier {
    Fn,
    Shift,
    Ctrl,
    Opt,
    Alt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardputerKey {
    Char(char),
    Esc,
    Backspace,
    Delete,
    Tab,
    Enter,
    Space,
    Up,
    Down,
    Left,
    Right,
    F(u8),
    Fn,
    Shift,
    Ctrl,
    Opt,
    Alt,
    BtnG0,
    BtnReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMatrixItem {
    pub normal: char,
    pub shift: char,
    pub fn_key: Option<CardputerKey>,
}

/// Official Cardputer 4x14 key definitions matching M5Stack's Keyboard.h
pub const KEY_MATRIX: [[KeyMatrixItem; COLS]; ROWS] = [
    // Row 0: ` 1 2 3 4 5 6 7 8 9 0 - = Del
    [
        KeyMatrixItem {
            normal: '`',
            shift: '~',
            fn_key: Some(CardputerKey::Esc),
        },
        KeyMatrixItem {
            normal: '1',
            shift: '!',
            fn_key: Some(CardputerKey::F(1)),
        },
        KeyMatrixItem {
            normal: '2',
            shift: '@',
            fn_key: Some(CardputerKey::F(2)),
        },
        KeyMatrixItem {
            normal: '3',
            shift: '#',
            fn_key: Some(CardputerKey::F(3)),
        },
        KeyMatrixItem {
            normal: '4',
            shift: '$',
            fn_key: Some(CardputerKey::F(4)),
        },
        KeyMatrixItem {
            normal: '5',
            shift: '%',
            fn_key: Some(CardputerKey::F(5)),
        },
        KeyMatrixItem {
            normal: '6',
            shift: '^',
            fn_key: Some(CardputerKey::F(6)),
        },
        KeyMatrixItem {
            normal: '7',
            shift: '&',
            fn_key: Some(CardputerKey::F(7)),
        },
        KeyMatrixItem {
            normal: '8',
            shift: '*',
            fn_key: Some(CardputerKey::F(8)),
        },
        KeyMatrixItem {
            normal: '9',
            shift: '(',
            fn_key: Some(CardputerKey::F(9)),
        },
        KeyMatrixItem {
            normal: '0',
            shift: ')',
            fn_key: Some(CardputerKey::F(10)),
        },
        KeyMatrixItem {
            normal: '-',
            shift: '_',
            fn_key: Some(CardputerKey::F(11)),
        },
        KeyMatrixItem {
            normal: '=',
            shift: '+',
            fn_key: Some(CardputerKey::F(12)),
        },
        KeyMatrixItem {
            normal: '\x08',
            shift: '\x08',
            fn_key: Some(CardputerKey::Delete),
        }, // Del / Backspace
    ],
    // Row 1: Tab q w e r t y u i o p [ ] \
    [
        KeyMatrixItem {
            normal: '\t',
            shift: '\t',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'q',
            shift: 'Q',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'w',
            shift: 'W',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'e',
            shift: 'E',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'r',
            shift: 'R',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 't',
            shift: 'T',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'y',
            shift: 'Y',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'u',
            shift: 'U',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'i',
            shift: 'I',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'o',
            shift: 'O',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'p',
            shift: 'P',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: '[',
            shift: '{',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: ']',
            shift: '}',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: '\\',
            shift: '|',
            fn_key: None,
        },
    ],
    // Row 2: Fn Shift a s d f g h j k l ; ' Enter
    [
        KeyMatrixItem {
            normal: '\0',
            shift: '\0',
            fn_key: Some(CardputerKey::Fn),
        },
        KeyMatrixItem {
            normal: '\0',
            shift: '\0',
            fn_key: Some(CardputerKey::Shift),
        },
        KeyMatrixItem {
            normal: 'a',
            shift: 'A',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 's',
            shift: 'S',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'd',
            shift: 'D',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'f',
            shift: 'F',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'g',
            shift: 'G',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'h',
            shift: 'H',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'j',
            shift: 'J',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'k',
            shift: 'K',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'l',
            shift: 'L',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: ';',
            shift: ':',
            fn_key: Some(CardputerKey::Up),
        },
        KeyMatrixItem {
            normal: '\'',
            shift: '"',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: '\n',
            shift: '\n',
            fn_key: None,
        }, // Enter
    ],
    // Row 3: Ctrl Opt Alt z x c v b n m , . / Space
    [
        KeyMatrixItem {
            normal: '\0',
            shift: '\0',
            fn_key: Some(CardputerKey::Ctrl),
        },
        KeyMatrixItem {
            normal: '\0',
            shift: '\0',
            fn_key: Some(CardputerKey::Opt),
        },
        KeyMatrixItem {
            normal: '\0',
            shift: '\0',
            fn_key: Some(CardputerKey::Alt),
        },
        KeyMatrixItem {
            normal: 'z',
            shift: 'Z',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'x',
            shift: 'X',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'c',
            shift: 'C',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'v',
            shift: 'V',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'b',
            shift: 'B',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'n',
            shift: 'N',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: 'm',
            shift: 'M',
            fn_key: None,
        },
        KeyMatrixItem {
            normal: ',',
            shift: '<',
            fn_key: Some(CardputerKey::Left),
        },
        KeyMatrixItem {
            normal: '.',
            shift: '>',
            fn_key: Some(CardputerKey::Down),
        },
        KeyMatrixItem {
            normal: '/',
            shift: '?',
            fn_key: Some(CardputerKey::Right),
        },
        KeyMatrixItem {
            normal: ' ',
            shift: ' ',
            fn_key: None,
        }, // Space
    ],
];

/// Returns the primary label and optional secondary (Shift/Fn) label to display on the key cap.
pub fn get_key_labels(row: usize, col: usize) -> (&'static str, &'static str, &'static str) {
    // (primary_text, shift_text, fn_text)
    match (row, col) {
        // Row 0
        (0, 0) => ("`", "~", "Esc"),
        (0, 1) => ("1", "!", "F1"),
        (0, 2) => ("2", "@", "F2"),
        (0, 3) => ("3", "#", "F3"),
        (0, 4) => ("4", "$", "F4"),
        (0, 5) => ("5", "%", "F5"),
        (0, 6) => ("6", "^", "F6"),
        (0, 7) => ("7", "&", "F7"),
        (0, 8) => ("8", "*", "F8"),
        (0, 9) => ("9", "(", "F9"),
        (0, 10) => ("0", ")", "F10"),
        (0, 11) => ("-", "_", "F11"),
        (0, 12) => ("=", "+", "F12"),
        (0, 13) => ("Del", "", "Del"),

        // Row 1
        (1, 0) => ("Tab", "", ""),
        (1, 1) => ("Q", "", ""),
        (1, 2) => ("W", "", ""),
        (1, 3) => ("E", "", ""),
        (1, 4) => ("R", "", ""),
        (1, 5) => ("T", "", ""),
        (1, 6) => ("Y", "", ""),
        (1, 7) => ("U", "", ""),
        (1, 8) => ("I", "", ""),
        (1, 9) => ("O", "", ""),
        (1, 10) => ("P", "", ""),
        (1, 11) => ("[", "{", ""),
        (1, 12) => ("]", "}", ""),
        (1, 13) => ("\\", "|", ""),

        // Row 2
        (2, 0) => ("Fn", "", ""),
        (2, 1) => ("Aa", "", ""),
        (2, 2) => ("A", "", ""),
        (2, 3) => ("S", "", ""),
        (2, 4) => ("D", "", ""),
        (2, 5) => ("F", "", ""),
        (2, 6) => ("G", "", ""),
        (2, 7) => ("H", "", ""),
        (2, 8) => ("J", "", ""),
        (2, 9) => ("K", "", ""),
        (2, 10) => ("L", "", ""),
        (2, 11) => (";", ":", "Up"),
        (2, 12) => ("'", "\"", ""),
        (2, 13) => ("Enter", "", ""),

        // Row 3
        (3, 0) => ("Ctrl", "", ""),
        (3, 1) => ("Opt", "", ""),
        (3, 2) => ("Alt", "", ""),
        (3, 3) => ("Z", "", ""),
        (3, 4) => ("X", "", ""),
        (3, 5) => ("C", "", ""),
        (3, 6) => ("V", "", ""),
        (3, 7) => ("B", "", ""),
        (3, 8) => ("N", "", ""),
        (3, 9) => ("M", "", ""),
        (3, 10) => (",", "<", "Left"),
        (3, 11) => (".", ">", "Down"),
        (3, 12) => ("/", "?", "Right"),
        (3, 13) => ("Space", "", ""),

        _ => ("", "", ""),
    }
}

/// Tracks the active keys, buttons, and state changes.
#[derive(Debug, Clone, Default)]
pub struct CardputerInputState {
    pub pressed_matrix_keys: HashSet<KeyCoord>,
    pub btn_g0_pressed: bool,
    pub btn_rst_pressed: bool,

    // Latch/modifier state
    pub fn_active: bool,
    pub shift_active: bool,
    pub ctrl_active: bool,
    pub opt_active: bool,
    pub alt_active: bool,

    // Events queued since last poll
    pub recent_chars: Vec<char>,
    /// Raw press/release transitions, including taps completed within one GUI frame.
    pub recent_matrix_events: Vec<(KeyCoord, bool)>,
    pub recent_keys: Vec<CardputerKey>,
    pub changed: bool,
}

impl CardputerInputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all pressed keys (e.g. on window blur/focus loss to prevent stuck keys)
    pub fn reset_all(&mut self) {
        let mut released: Vec<_> = self.pressed_matrix_keys.iter().copied().collect();
        released.sort_by_key(|c| (c.row, c.col));
        self.recent_matrix_events
            .extend(released.into_iter().map(|c| (c, false)));
        self.pressed_matrix_keys.clear();
        self.btn_g0_pressed = false;
        self.btn_rst_pressed = false;
        self.fn_active = false;
        self.shift_active = false;
        self.ctrl_active = false;
        self.opt_active = false;
        self.alt_active = false;
        self.changed = true;
    }

    /// Press a key using its matrix coordinates and optionally override the emitted key (e.g. for direct arrow navigation)
    pub fn press_key_override(&mut self, row: u8, col: u8, override_key: Option<CardputerKey>) {
        if (row as usize) >= ROWS || (col as usize) >= COLS {
            return;
        }
        let coord = KeyCoord::new(row, col);
        if !self.pressed_matrix_keys.insert(coord) {
            return; // Already pressed
        }
        self.changed = true;
        self.recent_matrix_events.push((coord, true));

        if let Some(key) = override_key {
            self.recent_keys.push(key);
            match key {
                CardputerKey::Enter => self.recent_chars.push('\n'),
                CardputerKey::Backspace | CardputerKey::Delete => self.recent_chars.push('\x08'),
                CardputerKey::Tab => self.recent_chars.push('\t'),
                CardputerKey::Space => self.recent_chars.push(' '),
                _ => {}
            }
            return;
        }

        // Check modifiers
        match (row, col) {
            (2, 0) => {
                self.fn_active = true;
                self.recent_keys.push(CardputerKey::Fn);
                return;
            }
            (2, 1) => {
                self.shift_active = true;
                self.recent_keys.push(CardputerKey::Shift);
                return;
            }
            (3, 0) => {
                self.ctrl_active = true;
                self.recent_keys.push(CardputerKey::Ctrl);
                return;
            }
            (3, 1) => {
                self.opt_active = true;
                self.recent_keys.push(CardputerKey::Opt);
                return;
            }
            (3, 2) => {
                self.alt_active = true;
                self.recent_keys.push(CardputerKey::Alt);
                return;
            }
            _ => {}
        }

        let item = &KEY_MATRIX[row as usize][col as usize];

        // Resolve Fn layer first if Fn is active
        if self.fn_active {
            if let Some(fn_key) = item.fn_key {
                self.recent_keys.push(fn_key);
                return;
            }
        }

        // Del key
        if row == 0 && col == 13 {
            self.recent_keys.push(CardputerKey::Backspace);
            self.recent_chars.push('\x08');
            return;
        }

        // Tab
        if row == 1 && col == 0 {
            self.recent_keys.push(CardputerKey::Tab);
            self.recent_chars.push('\t');
            return;
        }

        // Enter
        if row == 2 && col == 13 {
            self.recent_keys.push(CardputerKey::Enter);
            self.recent_chars.push('\n');
            return;
        }

        // Space
        if row == 3 && col == 13 {
            self.recent_keys.push(CardputerKey::Space);
            self.recent_chars.push(' ');
            return;
        }

        let ch = if self.shift_active {
            item.shift
        } else {
            item.normal
        };
        if ch != '\0' {
            self.recent_keys.push(CardputerKey::Char(ch));
            self.recent_chars.push(ch);
        }
    }

    /// Press a key using standard matrix behavior
    pub fn press_key(&mut self, row: u8, col: u8) {
        self.press_key_override(row, col, None);
    }

    /// Directly trigger a CardputerKey (e.g. for host navigation shortcuts or UI buttons)
    pub fn press_cardputer_key(&mut self, key: CardputerKey) {
        self.recent_keys.push(key);
        match key {
            CardputerKey::Enter => self.recent_chars.push('\n'),
            CardputerKey::Backspace | CardputerKey::Delete => self.recent_chars.push('\x08'),
            CardputerKey::Tab => self.recent_chars.push('\t'),
            CardputerKey::Space => self.recent_chars.push(' '),
            CardputerKey::Char(c) => self.recent_chars.push(c),
            _ => {}
        }
        self.changed = true;
    }

    pub fn release_key(&mut self, row: u8, col: u8) {
        let coord = KeyCoord::new(row, col);
        if self.pressed_matrix_keys.remove(&coord) {
            self.recent_matrix_events.push((coord, false));
            self.changed = true;
            match (row, col) {
                (2, 0) => self.fn_active = false,
                (2, 1) => self.shift_active = false,
                (3, 0) => self.ctrl_active = false,
                (3, 1) => self.opt_active = false,
                (3, 2) => self.alt_active = false,
                _ => {}
            }
        }
    }

    pub fn press_btn_g0(&mut self) {
        if !self.btn_g0_pressed {
            self.btn_g0_pressed = true;
            self.recent_keys.push(CardputerKey::BtnG0);
            self.changed = true;
        }
    }

    pub fn release_btn_g0(&mut self) {
        if self.btn_g0_pressed {
            self.btn_g0_pressed = false;
            self.changed = true;
        }
    }

    pub fn press_btn_rst(&mut self) {
        if !self.btn_rst_pressed {
            self.btn_rst_pressed = true;
            self.recent_keys.push(CardputerKey::BtnReset);
            self.changed = true;
        }
    }

    pub fn release_btn_rst(&mut self) {
        if self.btn_rst_pressed {
            self.btn_rst_pressed = false;
            self.changed = true;
        }
    }

    pub fn is_key_pressed(&self, row: u8, col: u8) -> bool {
        self.pressed_matrix_keys.contains(&KeyCoord::new(row, col))
    }

    pub fn take_matrix_events(&mut self) -> Vec<(KeyCoord, bool)> {
        std::mem::take(&mut self.recent_matrix_events)
    }

    /// Pulls recent characters, clearing the queue
    pub fn take_chars(&mut self) -> Vec<char> {
        std::mem::take(&mut self.recent_chars)
    }

    /// Pulls recent keys, clearing the queue
    pub fn take_keys(&mut self) -> Vec<CardputerKey> {
        std::mem::take(&mut self.recent_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_input_flow() {
        let mut input = CardputerInputState::new();

        // Type 'a'
        input.press_key(2, 2);
        assert!(input.is_key_pressed(2, 2));
        assert_eq!(input.take_chars(), vec!['a']);
        input.release_key(2, 2);
        assert!(!input.is_key_pressed(2, 2));

        // Shift + 'a' -> 'A'
        input.press_key(2, 1); // Shift
        input.press_key(2, 2); // 'a'
        assert_eq!(input.take_chars(), vec!['A']);
        input.release_key(2, 2);
        input.release_key(2, 1);

        // Fn + ';' -> Up arrow
        input.press_key(2, 0); // Fn
        input.press_key(2, 11); // ';'
        let keys = input.take_keys();
        assert!(keys.contains(&CardputerKey::Up));
        input.release_key(2, 11);
        input.release_key(2, 0);
    }

    #[test]
    fn test_reset_on_focus_loss() {
        let mut input = CardputerInputState::new();
        input.press_key(1, 1);
        input.press_key(2, 0);
        assert!(input.is_key_pressed(1, 1));
        assert!(input.fn_active);

        input.reset_all();
        assert!(!input.is_key_pressed(1, 1));
        assert!(!input.fn_active);
    }
}
