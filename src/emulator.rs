use crate::display::Display;
use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::input::InputModule;
use crate::instruction::{Instruction, ProcessorInstruction};
use crate::sound::SoundModule;
use crate::stack::Stack;
use anyhow::anyhow;
use log::{debug, info, trace, warn};
use rand::Rng;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

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
pub struct Emulator<D, S, I>
where
    D: Display,
    S: SoundModule,
    I: InputModule,
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
    /// The sound module for making sounds.
    sound_module: S,
    /// The module responsible for receiving user input.
    input_module: I,
    /// The stack of the emulator.
    stack: Stack<u16>,
    /// Holds the display data, each bit corresponds to a pixel.
    display_data: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    /// Tracks the last key pressed by the user.
    last_key_pressed: Option<u8>,
}

impl<D, S, I> Emulator<D, S, I>
where
    D: Display + 'static,
    S: SoundModule + 'static,
    I: InputModule + Clone + Send + 'static,
{
    /// Creates a new `Emulator` instance.
    ///
    pub fn new(display: D, sound_module: S, input_module: I) -> Emulator<D, S, I> {
        let mut emulator = Emulator {
            memory: [0; MEMORY_SIZE],
            registers: [0; NUMBER_OF_REGISTERS],
            index_register: 0,
            program_counter: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack_pointer: 0,
            stack: Stack::new(),
            display_data: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            display,
            sound_module,
            input_module,
            last_key_pressed: None,
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
        let mut tick_timer = Instant::now();
        let target_fps: u128 = 60;

        let (tx, rx): (Sender<u16>, Receiver<u16>) = mpsc::channel();
        let mut input_module_clone = self.input_module.clone();
        thread::spawn(move || loop {
            let key = input_module_clone.get_key_pressed();
            if let Some(some_key) = key {
                let _ = tx.send(some_key);
            }
        });
        // clear display
        self.display.clear();

        loop {
            let now = Instant::now();
            let elapsed_time = now.duration_since(tick_timer);
            let elapsed_ms = elapsed_time.as_millis();
            if elapsed_ms >= (1000 / target_fps) {
                self.handle_input(&rx);

                // Handle sound and delay timer.
                self.handle_timers();

                for _ in 0..=7 {
                    // fetch instruction & decode it
                    let instruction = self.fetch_instruction()?;
                    self.program_counter += 2;

                    // execute
                    self.execute_instruction(instruction)?;
                }

                tick_timer = Instant::now();
            } else {
                sleep(Duration::from_millis(1));
            }
        }
    }

    /// Handles the timers logic.
    fn handle_timers(&mut self) {
        // Handle timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        } else {
            self.do_beep()
        }
    }

    /// Handle the input
    fn handle_input(&mut self, receiver: &Receiver<u16>) {
        let received_input = receiver.try_recv();
        if let Ok(key_pressed) = received_input {
            if key_pressed == 0xFF {
                // Exit requested
                self.display.clear();
                println!("Thank you for playing! See you next time! :-)");
                std::process::exit(0);
            } else {
                self.last_key_pressed = Option::from((key_pressed & 0xF) as u8)
            }
        } else {
            self.last_key_pressed = None;
        }
    }

    /// Should make an audible beep.
    fn do_beep(&mut self) {
        self.sound_module.beep();
    }

    /// Executes the instruction
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), anyhow::Error> {
        match instruction.processor_instruction() {
            ProcessorInstruction::ClearScreen => {
                trace!("Clear display");
                self.display_data = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                self.display.clear()
            }
            ProcessorInstruction::Jump { address } => {
                trace!("Jump to address {:04x}", address);
                self.program_counter = address
            }
            ProcessorInstruction::SetRegister { register, data } => {
                trace!("Set register {} to data {:04x}", register, data);
                self.registers[register as usize] = data
            }
            ProcessorInstruction::AddValueToRegister { register, value } => {
                trace!("Add to register {} data {:04x}", register, value);
                let (result, _) = self.registers[register as usize].overflowing_add(value);
                self.registers[register as usize] = result;
            }
            ProcessorInstruction::SetIndexRegister { data } => {
                trace!("Set index register to data {:04x}", data);
                self.index_register = data;
            }
            ProcessorInstruction::Draw { vx, vy, rows } => {
                trace!("Draw vx_register={vx} vy_register={vy} pixels={rows}");
                let x_coordinate = self.registers[vx as usize];
                let y_coordinate = self.registers[vy as usize];

                // Keep track if any pixels were flipped
                let mut flipped = false;

                // Iterate over each row of our sprite
                for y_line in 0..rows {
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
            ProcessorInstruction::Call { address } => {
                trace!("Call {address:04x}");
                // Save PC to the stack
                self.stack.push(self.program_counter);
                // Set PC to subroutine address
                self.program_counter = address;
            }
            ProcessorInstruction::Set { vx, vy } => {
                trace!("Set VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] = self.registers[vy as usize];
            }
            ProcessorInstruction::BinaryOr { vx, vy } => {
                trace!("BinaryOr VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] |= self.registers[vy as usize]
            }
            ProcessorInstruction::BinaryAnd { vx, vy } => {
                trace!("BinaryAnd VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] &= self.registers[vy as usize]
            }
            ProcessorInstruction::BinaryXor { vx, vy } => {
                trace!("BinaryXor VX={vx:04x} VY={vy:04x}");
                self.registers[vx as usize] ^= self.registers[vy as usize]
            }
            ProcessorInstruction::Add { vx, vy } => {
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
            ProcessorInstruction::SubtractVX { vx, vy } => {
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
            ProcessorInstruction::SubtractVY { vx, vy } => {
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
            ProcessorInstruction::ShiftLeft { vx, vy } => {
                trace!("ShiftLeft VX={vx:04x} VY={vy:04x}");
                self.registers[0xF] = (self.registers[vx as usize] >> 7) & 1;
                self.registers[vx as usize] <<= 1;
            }
            ProcessorInstruction::ShiftRight { vx, vy } => {
                trace!("ShiftRight VX={vx:04x} VY={vy:04x}");
                self.registers[0xF] = self.registers[vx as usize] & 0x1;
                self.registers[vx as usize] >>= 1;
            }
            ProcessorInstruction::JumpWithOffset { address } => {
                let offset = self.registers[0x0];
                trace!("Jump With offset Address={address:04x} Offset={offset:04x}");

                self.program_counter = address + offset as u16
            }
            ProcessorInstruction::GenerateRandomNumber { vx, mask } => {
                trace!("Generate random number");
                self.registers[vx as usize] = rand::thread_rng().gen_range(0x00..0xFF) & mask
            }
            ProcessorInstruction::SkipEqualVXData { vx, data } => {
                trace!("SkipEqualVXData");
                let vx_data = self.registers[vx as usize];
                if vx_data == data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipNotEqualVXData { vx, data } => {
                trace!("SkipNotEqualVXData");
                let vx_data = self.registers[vx as usize];
                if vx_data != data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipEqualVXVY { vx, vy } => {
                trace!("SkipNotEqualVXData");
                let vx_data = self.registers[vx as usize];
                let vy_data = self.registers[vy as usize];
                if vx_data == vy_data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SkipNotEqualVXVY { vx, vy } => {
                trace!("SkipNotEqualVXVY");
                let vx_data = self.registers[vx as usize];
                let vy_data = self.registers[vy as usize];
                if vx_data != vy_data {
                    self.program_counter += 2
                }
            }
            ProcessorInstruction::SetVXToDelayTimer { vx } => {
                trace!("SetVXToDelayTimer");
                self.registers[vx as usize] = self.delay_timer
            }
            ProcessorInstruction::SetDelayTimer { vx } => {
                trace!("SetDelayTimer");
                self.delay_timer = self.registers[vx as usize]
            }
            ProcessorInstruction::SetSoundTimer { vx } => {
                trace!("SetSoundTimer");
                self.sound_timer = self.registers[vx as usize]
            }
            ProcessorInstruction::AddToIndex { vx } => {
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
            ProcessorInstruction::FontCharacter { vx } => {
                self.index_register = 0xF0 + (self.registers[vx as usize] as u16 & 0xF) * 5u16;
            }
            ProcessorInstruction::BinaryCodedDecimalConversion { vx } => {
                let number = self.registers[vx as usize];
                self.memory[self.index_register as usize] = number / 100;
                self.memory[self.index_register as usize + 1] = (number / 10) % 10;
                self.memory[self.index_register as usize + 2] = ((number) % 100) % 10;
            }
            ProcessorInstruction::LoadMemory { vx } => {
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.registers[i as usize] = self.memory[memory_index];
                }
            }
            ProcessorInstruction::StoreMemory { vx } => {
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.memory[memory_index] = self.registers[i as usize];
                }
            }
            ProcessorInstruction::GetKeyBlocking { vx } => {
                if let Some(key) = self.last_key_pressed {
                    self.registers[vx as usize] = key;
                } else {
                    self.program_counter -= 2;
                }
            }
            ProcessorInstruction::SkipIfKeyIsPressed { vx } => {
                if let Some(key) = self.last_key_pressed {
                    if self.registers[vx as usize] == key {
                        self.program_counter += 2;
                    }
                }
            }
            ProcessorInstruction::SkipIfKeyIsNotPressed { vx } => {
                if let Some(key) = self.last_key_pressed {
                    if self.registers[vx as usize] != key {
                        self.program_counter += 2;
                    }
                } else {
                    self.program_counter += 2;
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
    use crate::input::NoInput;
    use crate::sound::TerminalSound;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_load_font_data() {
        let emulator = Emulator::new(TerminalDisplay::new(), TerminalSound, NoInput);
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
        let mut emulator = Emulator::new(TerminalDisplay::new(), TerminalSound, NoInput);
        emulator
            .load_rom("roms/ibm-logo.ch8")
            .expect("failed to load ROM");

        // Assert
        assert_eq!(emulator.memory[0x200..0x200 + 132], rom_file_data)
    }
}
