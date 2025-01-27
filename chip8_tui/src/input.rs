use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

/// CrossTermInput implements input events via the crossterm crate.
#[derive(Clone)]
pub struct CrossTermInput {
    initialized: bool,
}

impl CrossTermInput {
    pub fn new() -> Self {
        enable_raw_mode().expect("failed to enable terminal raw mode.");
        CrossTermInput { initialized: true }
    }

    pub fn get_key_pressed(&mut self) -> Option<u16> {
        if !self.initialized {
            panic!("CrossTermInput needs to be constructed using ::new")
        }
        if let Ok(true) = poll(Duration::from_millis(25)) {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let read_result = read();

            if let Ok(Event::Key(key_event)) = read_result {
                match key_event.code {
                    KeyCode::Esc => {
                        return Some(0xFF);
                    }
                    KeyCode::Char(character) => {
                        if let Some(char) = character.to_lowercase().next() {
                            return match char {
                                '1' => Some(1),
                                '2' => Some(2),
                                '3' => Some(3),
                                '4' => Some(0xC),
                                'q' => Some(4),
                                'w' => Some(5),
                                'e' => Some(6),
                                'r' => Some(0xD),
                                'a' => Some(7),
                                's' => Some(8),
                                'd' => Some(9),
                                'f' => Some(0xE),
                                'z' => Some(0xA),
                                'x' => Some(0),
                                'c' => Some(0xB),
                                'v' => Some(0xF),
                                _ => None,
                            };
                        }
                    }
                    _ => {}
                }
            }

            return None;
        };
        None
    }
}

impl Default for CrossTermInput {
    fn default() -> Self {
        CrossTermInput::new()
    }
}

impl Drop for CrossTermInput {
    fn drop(&mut self) {
        disable_raw_mode().expect("failed to disable terminal raw mode.");
    }
}
