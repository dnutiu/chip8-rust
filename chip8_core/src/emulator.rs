use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::instruction::{Instruction, ProcessorInstruction};
use crate::stack::Stack;
use anyhow::anyhow;
use log::{debug, info, trace, warn};
use rand::Rng;
use std::io::Read;
use std::time::Instant;

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
pub struct Emulator {
    /// Memory represents the chip8_core's memory.
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
    /// The stack of the chip8_core.
    stack: Stack<u16>,
    /// Holds the display data, each bit corresponds to a pixel.
    display_data: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    /// Tracks the last key pressed by the user.
    last_key_pressed: Option<u8>,
    /// The target FPS on the emulator, default 60.
    target_fps: u128,
    /// Last tick
    last_tick_time: Option<Instant>,
}

impl Emulator {
    /// Creates a new `Emulator` instance.
    ///
    pub fn new() -> Emulator {
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
            last_key_pressed: None,
            target_fps: 60,
            last_tick_time: None,
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

    pub fn execute_and_fetch(&mut self) -> Result<(), anyhow::Error> {
        for _ in 0..=7 {
            // fetch instruction & decode it
            let instruction = self.fetch_instruction()?;
            self.program_counter += 2;

            // execute
            self.execute_instruction(instruction)?;
        }
        Ok(())
    }

    /// Handles the timers logic.
    fn handle_timers(&mut self) {
        // Handle timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }
    }

    /// should_beep will return true if the emulator should beep
    pub fn should_beep(&self) -> bool {
        self.sound_timer > 0
    }

    /// Tick ticks the timer.
    pub fn tick(&mut self) -> bool {
        let mut return_value = false;
        if self.last_tick_time.is_some() {
            let now = Instant::now();
            let elapsed_time = now.duration_since(self.last_tick_time.unwrap());
            let elapsed_ms = elapsed_time.as_millis();
            if elapsed_ms >= (1000 / self.target_fps) {
                self.last_tick_time = Some(Instant::now());
                // Handle sound and delay timer.
                self.handle_timers();
                return_value = true;
            }
        } else {
            self.last_tick_time = Some(Instant::now());
        }
        return_value
    }

    /// Handle the input
    pub fn handle_input(&mut self, key_pressed: Option<u16>) {
        if let Some(key_pressed) = key_pressed {
            if key_pressed == 0xFF {
                println!("Thank you for playing! See you next time! :-)");
                std::process::exit(0);
            } else {
                self.last_key_pressed = Option::from((key_pressed & 0xF) as u8)
            }
        } else {
            self.last_key_pressed = None;
        }
    }

    /// Executes the instruction
    fn execute_instruction(&mut self, instruction: Instruction) -> Result<(), anyhow::Error> {
        match instruction.processor_instruction() {
            ProcessorInstruction::ClearScreen => {
                trace!("Clear display");
                self.display_data = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
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
                trace!("FontCharacter");
                self.index_register = 0xF0 + (self.registers[vx as usize] as u16 & 0xF) * 5u16;
            }
            ProcessorInstruction::BinaryCodedDecimalConversion { vx } => {
                trace!("BinaryCodedDecimalConversion");
                let number = self.registers[vx as usize];
                self.memory[self.index_register as usize] = number / 100;
                self.memory[self.index_register as usize + 1] = (number / 10) % 10;
                self.memory[self.index_register as usize + 2] = ((number) % 100) % 10;
            }
            ProcessorInstruction::LoadMemory { vx } => {
                trace!("LoadMemory");
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.registers[i as usize] = self.memory[memory_index];
                }
            }
            ProcessorInstruction::StoreMemory { vx } => {
                trace!("StoreMemory");
                for i in 0..=vx {
                    let memory_index = (self.index_register + (i as u16)) as usize;
                    self.memory[memory_index] = self.registers[i as usize];
                }
            }
            ProcessorInstruction::GetKeyBlocking { vx } => {
                trace!("GetKeyBlocking");
                if let Some(key) = self.last_key_pressed {
                    self.registers[vx as usize] = key;
                } else {
                    self.program_counter -= 2;
                }
            }
            ProcessorInstruction::SkipIfKeyIsPressed { vx } => {
                trace!("SkipIfKeyIsPressed");
                if let Some(key) = self.last_key_pressed {
                    if self.registers[vx as usize] == key {
                        self.program_counter += 2;
                    }
                }
            }
            ProcessorInstruction::SkipIfKeyIsNotPressed { vx } => {
                trace!("SkipIfKeyIsNotPressed");
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

    /// Returns the display buffer of the emulator.
    pub fn get_display_buffer(&self) -> [bool; 2048] {
        self.display_data
    }

    /// Loads the ROM found at the rom path in the chip8_core's RAM memory.
    pub fn load_rom<T>(&mut self, mut rom: T) -> Result<(), anyhow::Error>
    where
        T: Read,
    {
        let amount = rom.read(&mut self.memory[0x200..])?;

        debug!("Loaded ROM of size {amount} into memory");

        // Set program counter to start of memory
        self.program_counter = 0x200;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs::File;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn test_load_font_data() {
        let emulator = Emulator::new();
        assert_eq!(emulator.memory[0xf0..0xf0 + 80], FONT_SPRITES)
    }

    #[test]
    fn test_load_rom_ibm_logo() {
        // Setup
        let mut file = File::open("../roms/ibm-logo.ch8").expect("Failed to test open ROM");
        let mut rom_file_data: [u8; 132] = [0; 132];
        file.read(&mut rom_file_data)
            .expect("Failed to read test ROM");

        let _ = file.seek(SeekFrom::Start(0));

        // Test
        let mut emulator = Emulator::new();
        emulator.load_rom(file).expect("failed to load ROM");

        // Assert
        assert_eq!(emulator.memory[0x200..0x200 + 132], rom_file_data)
    }

    #[test]
    fn test_execute_clear_screen_instruction() {
        // Setup
        let mut emulator = Emulator::new();
        for i in 10..30 {
            emulator.display_data[i] = true;
        }

        // Test
        emulator
            .execute_instruction(Instruction::new([0x00, 0xE0]))
            .expect("Failed to execute");

        // Assert
        assert!(emulator
            .display_data
            .iter()
            .all(|&pixel| { pixel == false }))
    }

    #[test]
    fn test_execute_jump() {
        let mut emulator = Emulator::new();

        emulator
            .execute_instruction(Instruction::new([0x1A, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 0xABC)
    }

    #[test]
    fn test_execute_set_register() {
        let mut emulator = Emulator::new();

        for i in 0x0..=0xF {
            let random_data: u8 = rand::thread_rng().gen_range(0x00..0xFF);
            emulator
                .execute_instruction(Instruction::new([0x60 + i, random_data]))
                .expect("Failed to execute");
            assert_eq!(emulator.registers[i as usize], random_data)
        }
    }

    #[test]
    fn test_execute_add_value_to_register() {
        let mut emulator = Emulator::new();

        emulator
            .execute_instruction(Instruction::new([0x71, 0xCC]))
            .expect("Failed to execute");
        emulator
            .execute_instruction(Instruction::new([0x75, 0xCC]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[1], 0xCC);
        assert_eq!(emulator.registers[5], 0xCC);

        emulator
            .execute_instruction(Instruction::new([0x75, 0xCC]))
            .expect("Failed to execute");
        assert_eq!(emulator.registers[5], 0x98);
    }

    #[test]
    fn test_execute_set_index_register() {
        let mut emulator = Emulator::new();

        emulator
            .execute_instruction(Instruction::new([0xAA, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.index_register, 0xABC);
    }

    #[test]
    fn test_execute_draw() {
        let mut emulator = Emulator::new();
        emulator.index_register = 0xF0;

        emulator
            .execute_instruction(Instruction::new([0xD0, 0x01]))
            .expect("Failed to execute");

        assert_eq!(
            emulator.display_data[0..=5],
            [true, true, true, true, false, false]
        )
    }

    #[test]
    fn test_execute_call() {
        let mut emulator = Emulator::new();
        emulator.program_counter = 0x200;

        emulator
            .execute_instruction(Instruction::new([0x2A, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 0xABC);
        assert_eq!(emulator.stack.peek().unwrap(), &0x200);
    }

    #[test]
    fn test_execute_return() {
        let mut emulator = Emulator::new();
        emulator.stack.push(0x269);

        emulator
            .execute_instruction(Instruction::new([0x00, 0xEE]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 0x269);
    }

    #[test]
    fn test_execute_set() {
        let mut emulator = Emulator::new();

        emulator
            .execute_instruction(Instruction::new([0x61, 0x40]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[1], 0x40);
    }

    #[test]
    fn test_execute_binary_or() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x6;
        emulator.registers[0xF] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xF1]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0xF);
    }

    #[test]
    fn test_execute_binary_and() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x6;
        emulator.registers[0xF] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xF2]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0x0);
    }

    #[test]
    fn test_execute_logical_xor() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0xF] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xF3]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0xE);
    }

    #[test]
    fn test_execute_logical_add() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0xF] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xF4]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0x10);
    }

    #[test]
    fn test_execute_logical_subtract_vx() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0xE] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xE5]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0xFE);
    }

    #[test]
    fn test_execute_logical_subtract_vy() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0xF] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0xF7]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0xFA);
    }

    #[test]
    fn test_execute_logical_shift_left() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0x2] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0x2E]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0xE);
    }

    #[test]
    fn test_execute_logical_shift_right() {
        let mut emulator = Emulator::new();
        emulator.registers[0x1] = 0x7;
        emulator.registers[0x2] = 0x9;

        emulator
            .execute_instruction(Instruction::new([0x81, 0x26]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0x1], 0x3);
    }

    #[test]
    fn test_execute_jump_with_offset() {
        let mut emulator = Emulator::new();
        emulator.registers[0x0] = 0x2;

        emulator
            .execute_instruction(Instruction::new([0xBA, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 0xABE);
    }

    #[test]
    fn test_execute_random_number() {
        let mut emulator = Emulator::new();

        emulator
            .execute_instruction(Instruction::new([0xCA, 0xBC]))
            .expect("Failed to execute");

        assert!(emulator.registers[0xA] <= 0xBC)
    }

    #[test]
    fn test_execute_skip_equal_vx_data() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xBC;

        emulator
            .execute_instruction(Instruction::new([0x3A, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_skip_not_equal_vx_data() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xB1;

        emulator
            .execute_instruction(Instruction::new([0x4A, 0xBC]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_skip_equal_vx_vy() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xBC;
        emulator.registers[0xB] = 0xBC;

        emulator
            .execute_instruction(Instruction::new([0x5A, 0xB0]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_skip_not_equal_vx_vy() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xBD;
        emulator.registers[0xB] = 0xBC;

        emulator
            .execute_instruction(Instruction::new([0x9A, 0xB0]))
            .expect("Failed to execute");

        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_set_vx_to_delay_timer() {
        let mut emulator = Emulator::new();
        emulator.delay_timer = 0xEE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x07]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0xA], 0xEE);
    }

    #[test]
    fn test_execute_set_delay_timer() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xEE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x15]))
            .expect("Failed to execute");

        assert_eq!(emulator.delay_timer, 0xEE);
    }

    #[test]
    fn test_execute_set_sound_timer() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xEE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x18]))
            .expect("Failed to execute");

        assert_eq!(emulator.sound_timer, 0xEE);
    }

    #[test]
    fn test_execute_add_to_index() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xEE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x1E]))
            .expect("Failed to execute");

        assert_eq!(emulator.index_register, 0xEE);
    }

    #[test]
    fn test_execute_get_font_character() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xEE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x29]))
            .expect("Failed to execute");

        assert_eq!(emulator.index_register, 0x136);
    }

    #[test]
    fn test_execute_bcd_convert() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0xFE;

        emulator
            .execute_instruction(Instruction::new([0xFA, 0x33]))
            .expect("Failed to execute");

        assert_eq!(emulator.memory[emulator.index_register as usize], 2);
        assert_eq!(emulator.memory[emulator.index_register as usize + 1], 5);
        assert_eq!(emulator.memory[emulator.index_register as usize + 2], 4);
    }

    #[test]
    fn test_execute_store_memory() {
        let mut emulator = Emulator::new();
        emulator.index_register = 0x20;
        for i in 0..0xF {
            emulator.registers[i] = (0xF + i) as u8;
        }

        emulator
            .execute_instruction(Instruction::new([0xFF, 0x55]))
            .expect("Failed to execute");

        for i in 0..0xF {
            assert_eq!(
                emulator.memory[(emulator.index_register + i) as usize],
                (0xF + i) as u8
            );
        }
    }

    #[test]
    fn test_execute_load_memory() {
        let mut emulator = Emulator::new();
        emulator.index_register = 0x20;
        for i in 0..0xF {
            emulator.memory[(emulator.index_register + i) as usize] = (0xF + i) as u8;
        }

        emulator
            .execute_instruction(Instruction::new([0xFF, 0x65]))
            .expect("Failed to execute");

        for i in 0..0xF {
            assert_eq!(emulator.registers[i], (0xF + i) as u8);
        }
    }

    #[test]
    fn test_execute_get_key_blocking() {
        let mut emulator = Emulator::new();
        emulator.program_counter = 0x10;

        emulator
            .execute_instruction(Instruction::new([0xFE, 0x0A]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 0xE);

        emulator.last_key_pressed = Some(0x1);
        emulator
            .execute_instruction(Instruction::new([0xFA, 0x0A]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 0xE);
        assert_eq!(emulator.registers[0xA], 0x1);
    }

    #[test]
    fn test_execute_skip_key_pressed() {
        let mut emulator = Emulator::new();
        emulator.program_counter = 0;
        emulator.registers[0xA] = 0x1;

        emulator
            .execute_instruction(Instruction::new([0xEA, 0x9E]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 0);

        emulator.last_key_pressed = Some(0x1);
        emulator
            .execute_instruction(Instruction::new([0xEA, 0x9E]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_skip_key_not_pressed() {
        let mut emulator = Emulator::new();
        emulator.program_counter = 0;
        emulator.registers[0xA] = 0x1;

        emulator
            .execute_instruction(Instruction::new([0xEA, 0xA1]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 2);

        emulator.last_key_pressed = Some(0x1);
        emulator
            .execute_instruction(Instruction::new([0xEA, 0xA1]))
            .expect("Failed to execute");
        assert_eq!(emulator.program_counter, 2);
    }

    #[test]
    fn test_execute_set_vx_to_vy() {
        let mut emulator = Emulator::new();
        emulator.registers[0xA] = 0;
        emulator.registers[0xB] = 0xEF;

        emulator
            .execute_instruction(Instruction::new([0x8A, 0xB0]))
            .expect("Failed to execute");

        assert_eq!(emulator.registers[0xA], 0xEF);
    }

    #[test]
    fn test_fetch_instruction() {
        let mut emulator = Emulator::new();
        emulator.program_counter = 0x200;
        emulator.memory[0x200] = 0x00;
        emulator.memory[0x201] = 0xEE;

        let instruction = emulator.fetch_instruction();
        match instruction {
            Ok(instruction) => {
                assert_eq!(instruction, 0x00EE);
            }
            Err(_) => {
                assert!(false, "Did not fetch");
            }
        }
    }

    #[test]
    fn test_handle_timers() {
        let mut emulator = Emulator::new();
        emulator.sound_timer = 10;
        emulator.delay_timer = 12;

        emulator.handle_timers();

        assert_eq!(emulator.sound_timer, 9);
        assert_eq!(emulator.delay_timer, 11);
    }

    #[test]
    fn test_handle_timers_beep() {
        // Given
        let mut emulator = Emulator::new();

        // Then
        emulator.sound_timer = 0;
        emulator.handle_timers();
        assert_eq!(emulator.should_beep(), false);

        emulator.sound_timer = 10;
        emulator.handle_timers();
        assert_eq!(emulator.should_beep(), true);
    }
}
