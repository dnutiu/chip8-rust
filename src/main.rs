use crate::display::RatatuiDisplay;
use crate::emulator::Emulator;
use env_logger;

mod display;
mod emulator;
mod input;
mod instruction;
mod stack;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut emulator = Emulator::new(RatatuiDisplay::new());

    emulator.emulate(String::from("./roms/1-chip8-logo.ch8"))?;

    Ok(())
}
