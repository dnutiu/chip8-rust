use anyhow::anyhow;
use log::{debug, info};
use std::fs::File;
use std::io::Read;
use std::path::{Path};
use crate::display::{Display};

const MEMORY_SIZE: usize = 4096;
const NUMBER_OF_REGISTERS: usize = 16;
const FONT_SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

/// Emulator emulates the Chip8 CPU.
pub struct Emulator<D> where D: Display {
    /// Memory represents the emulator's memory.
    memory: [u8; MEMORY_SIZE],
    /// Registers holds the general purpose registers.
    registers: [u8; NUMBER_OF_REGISTERS],
    /// The index register store memory addresses.
    index_register: u16,
    /// The program counter register tracks the currently executing instruction.
    program_counter: u16,
    /// The delay timer register. It is decremented at a rate of 60 Hz until it reaches 0.
    delay_timer: u8,
    /// The sound timer register. It is decremented at a rate of 60 Hz until it reaches 0.
    /// It plays a beeping sound when it's value is different from 0.
    sound_timer: u8,
    /// The stack pointer register.
    stack_pointer: u8,
    /// The display_data holds all the data associated with the display
    display: D
}

impl <D> Emulator<D> where D: Display {
    /// Creates a new `Emulator` instance.
    ///
    pub fn new(display: D) -> Emulator<D> {
        let mut emulator = Emulator {
            memory: [0; MEMORY_SIZE],
            registers: [0; NUMBER_OF_REGISTERS],
            display,
            index_register: 0,
            program_counter: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
        };

        emulator.load_font_data();

        emulator
    }

    fn load_font_data(&mut self) {
        info!("Loading font data...");
        FONT_SPRITES
            .iter()
            .enumerate()
            .for_each(|i| self.memory[0xf0 + i.0] = *i.1);
        info!("Loaded font data...");
        debug!("Memory:\n{}\n", format!("{:?}", self.memory))
    }

    /// Emulates the ROM specified at `path`.
    pub fn emulate<T>(&mut self, path: T) -> Result<(), anyhow::Error>
    where
        T: AsRef<Path> + std::fmt::Display,
    {
        self.load_rom(path)?;

        Ok(())
    }

    /// Loads the ROM found at the rom path in the emulator's RAM memory.
    fn load_rom<T>(&mut self, rom_file: T) -> Result<(), anyhow::Error>
    where
        T: AsRef<Path> + std::fmt::Display,
    {
        let mut file = File::open(&rom_file)?;

        // Check ROM length if it overflows max RAM size.
        let rom_size = file.metadata()?.len();
        debug!("Open ROM {} of size {} bytes.", &rom_file, rom_size);
        if rom_size > MEMORY_SIZE as u64 - 0x200 {
            return Err(anyhow!(
                "ROM at {} overflows emulator's RAM size of 4kB.",
                &rom_file
            ));
        }
        file.read(&mut self.memory[0x200..])?;

        debug!("Memory:\n{}\n", format!("{:?}", self.memory));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{TerminalDisplay};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_load_font_data() {
        let emulator = Emulator::new(TerminalDisplay::new());
        assert_eq!(emulator.memory[0xf0..0xf0 + 80], FONT_SPRITES)
    }

    #[test]
    fn test_load_rom_ibm_logo() {
        // Setup
        let mut file = File::open("roms/ibm-logo.ch8").expect("Failed to test open ROM");
        let mut rom_file_data: [u8; 132] = [0; 132];
        file.read(&mut rom_file_data)
            .expect("Failed to read test ROM");

        // Test
        let mut emulator = Emulator::new(TerminalDisplay::new());
        emulator
            .load_rom("roms/ibm-logo.ch8")
            .expect("failed to load ROM");

        // Assert
        assert_eq!(emulator.memory[0x200..0x200 + 132], rom_file_data)
    }
}
