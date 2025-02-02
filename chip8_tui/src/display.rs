use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Borders};
use ratatui::DefaultTerminal;

use chip8_core::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Simple terminal display for the Chip8's chip8_core.
pub struct TerminalDisplay {}

#[allow(dead_code)]
impl TerminalDisplay {
    pub fn new() -> TerminalDisplay {
        TerminalDisplay {}
    }

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
            println!()
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

    pub fn clear(&mut self) {
        self.terminal.clear().expect("Failed to clear terminal");
    }

    pub fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
        self.terminal
            .draw(|frame| {
                // Render the canvas widget
                frame.render_widget(
                    Block::default()
                        .title("Chip8 Emulator by nuculabs.dev")
                        .borders(Borders::ALL),
                    Rect::new(
                        0,
                        0,
                        (DISPLAY_WIDTH * 2 + 2) as u16,
                        (DISPLAY_HEIGHT * 2) as u16,
                    ),
                );
                display_data.iter().enumerate().for_each(|(index, pixel)| {
                    if *pixel {
                        let x = (index % DISPLAY_WIDTH) as u16;
                        let y = (index / DISPLAY_WIDTH) as u16;
                        let area = Rect::new(x * 2, y, 2, 1);
                        let block = Block::default().style(Style::new().on_white());
                        frame.render_widget(block, area);
                    }
                });
            })
            .expect("failed to draw");
    }
}

impl Drop for RatatuiDisplay {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
