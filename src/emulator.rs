use crate::display::Display;
use crate::instruction::{Instruction, ProcessorInstruction};
use crate::stack::Stack;
use anyhow::anyhow;
use log::{debug, info, trace, warn};
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::Instant;
use std::{thread, time};

/// Represents the display's width in pixels.
const DISPLAY_WIDTH: usize = 64;

/// Represents the display's height pixels.
const DISPLAY_HEIGHT: usize = 32;

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
pub struct Emulator<D>
where
    D: Display,
{
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
    display: D,
    /// The stack of the emulator.
    stack: Stack<u16>,
    /// Holds the display data, each bit corresponds to a pixel.
    display_data: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl<D> Emulator<D>
where
    D: Display,
{
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
            stack: Stack::new(),
            display_data: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
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
        info!("Loaded font data into memory at 0xf0.");
    }

    /// Emulates the ROM specified at `path`.
    pub fn emulate<T>(&mut self, path: T) -> Result<(), anyhow::Error>
    where
        T: AsRef<Path> + std::fmt::Display,
    {
        self.load_rom(path)?;
        self.emulation_loop::<T>()?;
        Ok(())
    }

    /// Emulation loop executes the fetch -> decode -> execute pipeline
    fn emulation_loop<T>(&mut self) -> Result<(), anyhow::Error> {
        let mut start_time = Instant::now();
        let mut last_program_counter = self.program_counter;
        loop {
            self.handle_timers(&mut start_time);

            // fetch instruction & decode it
            let instruction = self.fetch_instruction()?;
            self.program_counter += 2;

            if last_program_counter != self.program_counter {
                debug!("PC={} {:04x}", self.program_counter, instruction);
            }
            last_program_counter = self.program_counter;

            // execute
            self.execute_instruction(instruction)?;

            // insert some delay
            thread::sleep(time::Duration::from_millis(10));
        }
    }

    /// Handles the timers logic.
    fn handle_timers(&mut self, start_time: &mut Instant) {
        // Handle 60hz timers
        let elapsed_time = start_time.elapsed().as_micros();
        // 16667 us which is 1/60 of a second
        if elapsed_time > 16667 {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            } else {
                self.do_beep()
            }
            *start_time = Instant::now()
        }
    }

    /// Should make an audible beep.
    fn do_beep(&mut self) {
        // beep, for now
    }

    /// Executes the instruction
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), anyhow::Error> {
        match instruction.processor_instruction() {
            ProcessorInstruction::ClearScreen => {
                trace!("Clear display");
                self.display.clear()
            }
            ProcessorInstruction::Jump(address) => {
                trace!("Jump to address {:04x}", address);
                self.program_counter = address
            }
            ProcessorInstruction::SetRegister(register, data) => {
                trace!("Set register {} to data {:04x}", register, data);
                self.registers[register as usize] = data
            }
            ProcessorInstruction::AddValueToRegister(register, data) => {
                trace!("Add to register {} data {:04x}", register, data);
                let (result, _) = self.registers[register as usize].overflowing_add(data);
                self.registers[register as usize] = result;
            }
            ProcessorInstruction::SetIndexRegister(data) => {
                trace!("Set index register to data {:04x}", data);
                self.index_register = data;
            }
            ProcessorInstruction::Draw(vx_register, vy_register, num_rows) => {
                trace!(
                    "Draw vx_register={vx_register} vy_register={vy_register} pixels={num_rows}"
                );
                let x_coordinate = self.registers[vx_register as usize];
                let y_coordinate = self.registers[vy_register as usize];

                // Keep track if any pixels were flipped
                let mut flipped = false;

                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.index_register + y_line as u16;
                    let pixels = self.memory[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coordinate + x_line) as usize % DISPLAY_WIDTH;
                            let y = (y_coordinate + y_line) as usize % DISPLAY_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let index = x + DISPLAY_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.display_data[index];
                            self.display_data[index] ^= true;
                        }
                    }
                }

                if flipped {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }

                self.display.render(&self.display_data);
            }
            ProcessorInstruction::Return => {
                let value = self.stack.pop().unwrap();
                trace!("Return to {value:04x}");
                self.program_counter = value;
            }
            ProcessorInstruction::Call(address) => {
                trace!("Call {address:04x}");
                // Save PC to the stack
                self.stack.push(self.program_counter);
                // Set PC to subroutine address
                self.program_counter = address;
            }
            ProcessorInstruction::Set(vx, vy) => {
                trace!("Set VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] = self.registers[vy as usize];
            }
            ProcessorInstruction::BinaryOr(vx, vy) => {
                trace!("BinaryOr VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] |= self.registers[vy as usize]
            }
            ProcessorInstruction::BinaryAnd(vx, vy) => {
                trace!("BinaryAnd VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] &= self.registers[vy as usize]
            }
            ProcessorInstruction::BinaryXor(vx, vy) => {
                trace!("BinaryXor VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] ^= self.registers[vy as usize]
            }
            ProcessorInstruction::Add(vx, vy) => {
                trace!("Add VX={vx:04x} VY={vy:04x}");
                let (result, overflow) =
                    self.registers[vx as usize].overflowing_add(self.registers[vy as usize]);

                self.registers[vx as usize] = result;
                if overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            ProcessorInstruction::SubtractVX(vx, vy) => {
                trace!("SubtractVX VX={vx:04x} VY={vy:04x}");
                if self.registers[vx as usize] > self.registers[vy as usize] {
                    self.registers[0xF] = 1
                } else {
                    // The register 0xF will be 0 if there's an underflow.
                    self.registers[0xF] = 0
                }
                let (result, _) =
                    self.registers[vx as usize].overflowing_sub(self.registers[vy as usize]);
                self.registers[vx as usize] = result;
            }
            ProcessorInstruction::SubtractVY(vx, vy) => {
                trace!("SubtractVY VX={vx:04x} VY={vy:04x}");
                if self.registers[vy as usize] > self.registers[vx as usize] {
                    self.registers[0xF] = 1
                } else {
                    // The register 0xF will be 0 if there's an underflow.
                    self.registers[0xF] = 0
                }
                let (result, _) =
                    self.registers[vy as usize].overflowing_sub(self.registers[vx as usize]);
                self.registers[vx as usize] = result;
            }
            ProcessorInstruction::ShiftLeft(vx, vy) => {
                trace!("ShiftLeft VX={vx:04x} VY={vy:04x}");
                self.registers[0xF] = (self.registers[vx as usize] >> 7) & 1;
                self.registers[vx as usize] <<= 1;
            }
            ProcessorInstruction::ShiftRight(vx, vy) => {
                trace!("ShiftRight VX={vx:04x} VY={vy:04x}");
                self.registers[0xF] = self.registers[vx as usize] & 0x1;
                self.registers[vx as usize] >>= 1;
            }
            ProcessorInstruction::JumpWithOffset(address) => {
                let offset = self.registers[0x0];
                trace!("Jump With offset Address={address:04x} Offset={offset:04x}");

                self.program_counter = address + offset as u16
            }
            ProcessorInstruction::GenerateRandomNumber(register, data) => {
                trace!("Generate random number");
                self.registers[register as usize] = rand::thread_rng().gen_range(0x00..0xFF) & data
            }
            ProcessorInstruction::SkipEqualVXData(vx, data) => {
                trace!("SkipEqualVXData");
                let vx_data = self.registers[vx as usize];
                if vx_data == data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipNotEqualVXData(vx, data) => {
                trace!("SkipNotEqualVXData");
                let vx_data = self.registers[vx as usize];
                if vx_data != data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipEqualVXVY(vx, vy) => {
                trace!("SkipNotEqualVXData");
                let vx_data = self.registers[vx as usize];
                let vy_data = self.registers[vy as usize];
                if vx_data == vy_data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipNotEqualVXVY(vx, vy) => {
                trace!("SkipNotEqualVXVY");
                let vx_data = self.registers[vx as usize];
                let vy_data = self.registers[vy as usize];
                if vx_data != vy_data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SetVXToDelayTimer(vx) => {
                trace!("SetVXToDelayTimer");
                self.registers[vx as usize] = self.delay_timer
            }
            ProcessorInstruction::SetDelayTimer(vx) => {
                trace!("SetDelayTimer");
                self.delay_timer = self.registers[vx as usize]
            }
            ProcessorInstruction::SetSoundTimer(vx) => {
                trace!("SetSoundTimer");
                self.sound_timer = self.registers[vx as usize]
            }
            ProcessorInstruction::AddToIndex(vx) => {
                trace!("AddToIndex");
                let (result, overflow) = self
                    .index_register
                    .overflowing_add(self.registers[vx as usize] as u16);
                self.index_register = result;
                if overflow {
                    self.registers[0xF] = 1
                } else {
                    self.registers[0xF] = 0
                }
            }
            ProcessorInstruction::FontCharacter(vx) => {
                self.index_register = 0xF0 + (self.registers[vx as usize] & 0x0F) as u16;
            }
            ProcessorInstruction::BinaryCodedDecimalConversion(vx) => {
                self.memory[self.index_register as usize] = vx / 100;
                self.memory[self.index_register as usize + 1] = (vx / 10) % 10;
                self.memory[self.index_register as usize + 2] = ((vx) % 100) % 10;
            }
            ProcessorInstruction::LoadMemory(vx) => {
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.registers[i as usize] = self.memory[memory_index];
                }
            }
            ProcessorInstruction::StoreMemory(vx) => {
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.memory[memory_index] = self.registers[i as usize];
                }
            }
            _ => {
                warn!("Unknown instruction: {:04x}, skipping.", instruction);
            }
        }
        Ok(())
    }

    /// Fetches the current instruction from the memory without incrementing the program counter.
    fn fetch_instruction(&self) -> Result<Instruction, anyhow::Error> {
        if self.program_counter as usize >= self.memory.len() {
            return Err(anyhow!("program_counter is out of range"));
        }

        Ok(Instruction::new([
            self.memory[self.program_counter as usize],
            self.memory[self.program_counter as usize + 1],
        ]))
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

        // Set program counter to start of memory
        self.program_counter = 0x200;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::TerminalDisplay;
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
