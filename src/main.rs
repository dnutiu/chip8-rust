use crate::display::{RatatuiDisplay};
use crate::emulator::Emulator;
use env_logger;

mod display;
mod emulator;
mod instruction;
mod stack;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut emulator = Emulator::new(RatatuiDisplay::new());

    emulator.emulate(String::from("./roms/ibm-logo.ch8"))?;

    Ok(())
}
