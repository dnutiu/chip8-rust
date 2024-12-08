use crate::display::RatatuiDisplay;
use crate::emulator::Emulator;
use crate::sound::TerminalSound;
use env_logger;

mod display;
mod emulator;
mod input;
mod instruction;
mod sound;
mod stack;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut emulator = Emulator::new(RatatuiDisplay::new(), TerminalSound);

    emulator.emulate(String::from("./roms/3-corax+.ch8"))?;

    Ok(())
}
