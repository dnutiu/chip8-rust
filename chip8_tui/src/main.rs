mod display;
mod input;

use crate::display::RatatuiDisplay;
use crate::input::CrossTermInput;
use clap::Parser;
use chip8_core::emulator::{tick, Emulator};
use chip8_core::read::StdFileReader;
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;

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

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let args = CliArgs::parse();

    let file = File::open(&args.rom_path)?;

    let mut emulator = Emulator::new();
    let mut display = RatatuiDisplay::new();
    let mut input = CrossTermInput::new();
    emulator.load_rom(StdFileReader::new(file))?;

    display.clear();

    let mut last_tick_time = None;
    loop {
        if tick(&mut last_tick_time) {
            emulator.handle_input(input.get_key_pressed());

            if emulator.should_beep() {
                print!("\x07");
            }

            emulator.execute_and_fetch()?;

            // render
            display.render(&emulator.get_display_buffer());
        } else {
            sleep(Duration::from_millis(1));
        }
    }
}
