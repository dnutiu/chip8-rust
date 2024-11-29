/// Represents the display's width in pixels.
const DISPLAY_WIDTH: usize = 64;

/// Represents the display's height pixels.
const DISPLAY_HEIGHT: usize = 32;

/// Display trait
pub trait Display {
    /// Re-draws the display.
    fn redraw(&self);
    /// Draws the display.
    fn draw(&mut self);
}

/// Display models the Chip8's display.
pub struct TerminalDisplay {
    /// Holds the display data, each bit corresponds to a pixel.
    display_data: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl TerminalDisplay {
    pub fn new() -> TerminalDisplay {
        TerminalDisplay {
            display_data: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        }
    }
}

impl Display for TerminalDisplay {
    /// Re-draws the display.
    fn redraw(&self) {
        // ANSI Escape code to move cursor to row 1 column 1
        // See https://stackoverflow.com/a/4062051
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }
    /// Draws the display.
    fn draw(&mut self) {
        for row in 0..32 {
            for column in 0..64 {
                if self.display_data[row * DISPLAY_WIDTH + column] {
                    print!("█")
                } else {
                    print!(" ")
                }
            }
            print!("\n")
        }
    }
}
