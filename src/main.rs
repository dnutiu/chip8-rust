use crate::emulator::Emulator;
use env_logger;
use crate::display::TerminalDisplay;

mod emulator;
mod display;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut emulator = Emulator::new(TerminalDisplay::new());

    emulator.emulate(String::from("./roms/ibm-logo.ch8"))?;

    Ok(())
}
