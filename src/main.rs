use crate::emulator::Emulator;
use env_logger;

mod emulator;

fn main() {
    env_logger::init();

    let emulator = Emulator::new();
}
