use crate::emulator::Emulator;
use env_logger;

mod emulator;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut emulator = Emulator::new();

    emulator.emulate(String::from("./roms/ibm-logo.ch8"))?;

    Ok(())
}
