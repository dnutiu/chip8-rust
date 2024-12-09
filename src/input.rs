use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::enable_raw_mode;
use std::time::Duration;

/// InputModule retrieves the keys from the hardware or software input control module.
pub trait InputModule {
    /// Returns the key value of the corresponding pressed key.
    /// None if no key is pressed.
    fn get_key_pressed(&mut self) -> Option<u8>;
}

/// NoInput always returns none when queried for input.
pub struct NoInput;

impl InputModule for NoInput {
    fn get_key_pressed(&mut self) -> Option<u8> {
        None
    }
}

/// CrossTermInput implements input events via the crossterm crate.
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
    fn get_key_pressed(&mut self) -> Option<u8> {
        if !self.initialized {
            panic!("CrossTermInput needs to be constructed using ::new")
        }
        if let Ok(true) = poll(Duration::from_millis(2)) {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let read_result = read();

            if let Ok(event) = read_result {
                match event {
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char(character) => {
                            let lowercase_character = character.to_lowercase();
                            for char in lowercase_character {
                                match char {
                                    '1' => {
                                        return Some(1);
                                    }
                                    '2' => {
                                        return Some(2);
                                    }
                                    '3' => {
                                        return Some(3);
                                    }
                                    '4' => {
                                        return Some(0xC);
                                    }
                                    'q' => {
                                        return Some(4);
                                    }
                                    'w' => {
                                        return Some(5);
                                    }
                                    'e' => {
                                        return Some(6);
                                    }
                                    'r' => {
                                        return Some(0xD);
                                    }
                                    'a' => {
                                        return Some(7);
                                    }
                                    's' => {
                                        return Some(8);
                                    }
                                    'd' => {
                                        return Some(9);
                                    }
                                    'f' => {
                                        return Some(0xE);
                                    }
                                    'z' => {
                                        return Some(0xA);
                                    }
                                    'x' => {
                                        return Some(0);
                                    }
                                    'c' => {
                                        return Some(0xB);
                                    }
                                    'v' => {
                                        return Some(0xF);
                                    }
                                    _ => {}
                                }
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
