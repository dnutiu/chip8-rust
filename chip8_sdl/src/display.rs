use crate::{BACKGROUND_COLOR, PIXEL_COLOR};
use anyhow::anyhow;
use chip8_core::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use log::error;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

/// SDL2 display module for the Chip8 emulator.
pub struct SdlDisplay {
    canvas: WindowCanvas,
}

impl SdlDisplay {
    pub fn new(sdl_context: &Sdl) -> Result<Self, anyhow::Error> {
        let video_subsystem = sdl_context.video().map_err(|s| anyhow!(s))?;

        let window = video_subsystem
            .window("Chip8 Emulator by nuculabs.dev", 816, 648)
            .vulkan()
            .build()
            .map_err(|e| e.to_string())
            .map_err(|s| anyhow!(s))?;

        let canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .map_err(|s| anyhow!(s))?;

        Ok(SdlDisplay { canvas })
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();
    }

    pub fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
        for row in 0..32 {
            for column in 0..64 {
                if display_data[row * DISPLAY_WIDTH + column] {
                    self.canvas.set_draw_color(PIXEL_COLOR);
                    let result = self.canvas.fill_rect(Rect::new(
                        column as i32 * 12 + 24,
                        row as i32 * 18 + 18,
                        12,
                        18,
                    ));
                    if let Err(error_message) = result {
                        error!("{}", error_message)
                    }
                } else {
                    self.canvas.set_draw_color(BACKGROUND_COLOR);
                    let result = self.canvas.fill_rect(Rect::new(
                        column as i32 * 12 + 24,
                        row as i32 * 18 + 18,
                        12,
                        18,
                    ));
                    if let Err(error_message) = result {
                        error!("{}", error_message)
                    }
                }
            }
        }
        self.canvas.present()
    }
}
