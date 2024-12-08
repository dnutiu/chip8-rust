use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{Block, Borders};
use ratatui::DefaultTerminal;

/// Represents the display's width in pixels.
const DISPLAY_WIDTH: usize = 64;

/// Represents the display's height pixels.
const DISPLAY_HEIGHT: usize = 32;

/// Display trait
pub trait Display {
    /// Re-draws the display.
    fn clear(&mut self);
    /// Renders the display data on screen.
    fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]);
}

/// Simple terminal display for the Chip8's emulator.
pub struct TerminalDisplay {}

impl TerminalDisplay {
    pub fn new() -> TerminalDisplay {
        TerminalDisplay {}
    }
}

impl Display for TerminalDisplay {
    /// Re-draws the display.
    fn clear(&mut self) {
        // ANSI Escape code to move cursor to row 1 column 1
        // See https://stackoverflow.com/a/4062051
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
    }
    /// Renders the display data on screen.
    fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
        for row in 0..32 {
            for column in 0..64 {
                if display_data[row * DISPLAY_WIDTH + column] {
                    print!("█")
                } else {
                    print!(" ")
                }
            }
            print!("\n")
        }
    }
}

/// Ratatui based TUI display.
pub struct RatatuiDisplay {
    terminal: DefaultTerminal,
}

impl RatatuiDisplay {
    pub fn new() -> RatatuiDisplay {
        RatatuiDisplay {
            terminal: ratatui::init(),
        }
    }
}

impl Display for RatatuiDisplay {
    fn clear(&mut self) {
        self.terminal.clear().expect("Failed to clear terminal");
    }

    fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
        self.terminal
            .draw(|frame| {
                let canvas = Canvas::default()
                    .block(
                        Block::default()
                            .title("Chip8 Emulator by nuculabs.dev")
                            .borders(Borders::ALL),
                    )
                    .marker(Marker::Braille)
                    .paint(|ctx| {
                        for row in 0..DISPLAY_HEIGHT {
                            for column in 0..DISPLAY_WIDTH {
                                if display_data[row * DISPLAY_WIDTH + column] {
                                    ctx.draw(&ratatui::widgets::canvas::Rectangle {
                                        x: column as f64,
                                        y: DISPLAY_HEIGHT as f64 - row as f64,
                                        width: 1.0,
                                        height: 1.0,
                                        color: Color::White,
                                    });
                                }
                            }
                        }
                    })
                    .x_bounds([0.0, DISPLAY_WIDTH as f64])
                    .y_bounds([0.0, DISPLAY_HEIGHT as f64]);

                // Render the canvas widget
                frame.render_widget(canvas, frame.area());
            })
            .expect("failed to draw");
    }
}
