use crate::display::RatatuiDisplay;
use crate::emulator::Emulator;
use crate::input::CrossTermInput;
use crate::sound::TerminalSound;
use env_logger;
use std::env;
use std::path::PathBuf;

mod display;
mod emulator;
mod input;
mod instruction;
mod sound;
mod stack;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let rom_path = PathBuf::from(env::args().skip(1).next().expect("rom path not provided."));

    let mut emulator = Emulator::new(RatatuiDisplay::new(), TerminalSound, CrossTermInput::new());
    emulator.emulate(rom_path.to_str().unwrap())?;

    Ok(())
}
