use anyhow::anyhow;
use log::{debug, info};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const MEMORY_SIZE: usize = 4096;
const NUMBER_OF_REGISTERS: usize = 16;

/// Emulator emulates the Chip8 CPU.
pub struct Emulator {
    memory: [u8; MEMORY_SIZE],
    registers: [u16; NUMBER_OF_REGISTERS],
    program_counter: u16,
}

impl Emulator {
    /// Creates a new `Emulator` instance.
    ///
    pub fn new() -> Emulator {
        let mut emulator = Emulator {
            memory: [0; 4096],
            registers: [0; 16],
            program_counter: 0,
        };

        emulator.load_font_data();

        emulator
    }

    fn load_font_data(&mut self) {
        info!("Loading font data...");
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
