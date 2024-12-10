use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

/// InputModule retrieves the keys from the hardware or software input control module.
pub trait InputModule {
    /// Returns the key value of the corresponding pressed key.
    /// None if no key is pressed.
    fn get_key_pressed(&mut self) -> Option<u16>;
}

/// NoInput always returns none when queried for input.
#[derive(Clone)]
pub struct NoInput;

impl InputModule for NoInput {
    fn get_key_pressed(&mut self) -> Option<u16> {
        None
    }
}

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
}

impl Default for CrossTermInput {
    fn default() -> Self {
        CrossTermInput::new()
    }
}

impl InputModule for CrossTermInput {
    fn get_key_pressed(&mut self) -> Option<u16> {
        if !self.initialized {
            panic!("CrossTermInput needs to be constructed using ::new")
        }
        if let Ok(true) = poll(Duration::from_millis(100)) {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let read_result = read();

            if let Ok(event) = read_result {
                match event {
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Esc => {
                            return Some(0xFF);
                        }
                        KeyCode::Char(character) => {
                            let lowercase_character = character.to_lowercase();
                            for char in lowercase_character {
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
                    },
                    // ignore non key events
                    _ => {}
                }
            }

            return None;
        };
        None
    }
}

impl Drop for CrossTermInput {
    fn drop(&mut self) {
        disable_raw_mode().expect("failed to disable terminal raw mode.");
    }
}
