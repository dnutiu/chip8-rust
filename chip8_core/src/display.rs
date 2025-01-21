/// Represents the display's width in pixels.
pub const DISPLAY_WIDTH: usize = 64;

/// Represents the display's height pixels.
pub const DISPLAY_HEIGHT: usize = 32;

/// Display trait
pub trait Display {
    /// Re-draws the display.
    fn clear(&mut self);
    /// Renders the display data on screen.
    fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
}

/// Simple display module for testing. Does nothing.
pub(crate) struct TestingDisplay;

impl Display for TestingDisplay {
    fn clear(&mut self) {}

    fn render(&mut self, _display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {}
}
