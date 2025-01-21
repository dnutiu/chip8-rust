use anyhow::anyhow;
use clap::Parser;
use emulator::display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use emulator::emulator::Emulator;
use emulator::input::InputModule;
use emulator::sound::SoundModule;
use log::error;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
use std::fs::File;

const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const PIXEL_COLOR: Color = Color::RGB(0, 255, 0);

#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    about = "A Chip8 chip8_core.",
    long_about = "A program which emulates the Chip8 system."
)]
struct CliArgs {
    /// The path to the ROM file to emulate.
    rom_path: String,
}

/// SDL2 display module for the Chip8 emulator.
struct SDLDisplayBackend {
    canvas: WindowCanvas,
}

impl SDLDisplayBackend {
    fn new(sdl_context: &Sdl) -> Result<Self, anyhow::Error> {
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

        Ok(SDLDisplayBackend { canvas })
    }
}

impl Display for SDLDisplayBackend {
    fn clear(&mut self) {
        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();
    }

    fn render(&mut self, display_data: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
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

struct DummySound;

impl SoundModule for DummySound {
    fn beep(&mut self) {}
}

#[derive(Clone)]
struct DummyInputHandler;

impl InputModule for DummyInputHandler {
    fn get_key_pressed(&mut self) -> Option<u16> {
        return None;
    }
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();

    let file = File::open(&args.rom_path)?;

    let sdl_context = sdl2::init().map_err(|s| anyhow!(s))?;

    let sdl_display_backend: SDLDisplayBackend = SDLDisplayBackend::new(&sdl_context)?;

    let mut emulator = Emulator::new(sdl_display_backend, DummySound, DummyInputHandler);
    emulator.emulate(file)?;

    // let event_pump = sdl_context.event_pump().map_err(|s| anyhow!(s))?;

    // Show it on the screen
    // canvas.present();
    // let mut index = 0;
    // 'main: loop {
    //     index += 10;
    //     canvas.fill_rect(Rect::new(index, 100, 10, 10))?;
    //     canvas.present();
    //     thread::sleep(Duration::new(1, 0));
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => break 'main,
    //             _ => {}
    //         }
    //     }
    // }

    Ok(())
}
