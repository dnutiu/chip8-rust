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
