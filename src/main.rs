use crate::display::RatatuiDisplay;
use crate::emulator::Emulator;
use crate::input::CrossTermInput;
use crate::sound::TerminalSound;
use clap::Parser;
use std::fs::File;

mod display;
mod emulator;
mod input;
mod instruction;
mod sound;
mod stack;

#[derive(Parser, Debug)]
#[command(
    version = "1.0",
    about = "A Chip8 emulator.",
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

    let mut emulator = Emulator::new(RatatuiDisplay::new(), TerminalSound, CrossTermInput::new());
    emulator.emulate(file)?;

    Ok(())
}
