use std::fmt;
use std::fmt::{Display, Formatter, LowerHex};
/*
Although every instruction will have a first nibble that tells you what kind of instruction it is,
the rest of the nibbles will have different meanings. To differentiate these meanings,
we usually call them different things, but all of them can be any hexadecimal number from 0 to F:

    X: The second nibble. Used to look up one of the 16 registers (VX) from V0 through VF.
    Y: The third nibble. Also used to look up one of the 16 registers (VY) from V0 through VF.
    N: The fourth nibble. A 4-bit number.
    NN: The second byte (third and fourth nibbles). An 8-bit immediate number.
    NNN: The second, third and fourth nibbles. A 12-bit immediate memory address.
 */

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProcessorInstruction {
    /// Clears the screen
    ClearScreen,
    /// Jumps to a given address
    Jump(u16),
    /// Sets the register in the first argument to the given value
    SetRegister(u8, u8),
    /// Adds the value to the register
    AddValueToRegister(u8, u8),
    /// Sets the index register
    SetIndexRegister(u16),
    /// Draws to the screen.
    Draw(u8, u8, u8),
    /// Call sets PC to the address and saves the return address on the stack
    Call(u16),
    /// Pops the stack and sets the PC
    Return,
    /// Set VX to the value of VY
    Set(u8, u8),
    /// Or VX with VY and store in VX.
    BinaryOr(u8, u8),
    /// And VX with VY and store in VX.
    BinaryAnd(u8, u8),
    /// XOR VX with VY and store in VX.
    BinaryXor(u8, u8),
    /// Add VX with VY and store in VX, if addition overflows set the carry flag 0xVF
    Add(u8, u8),
    /// Subtract VX from VY and set to VX. VX = VX - VY. Affects the carry flag.
    SubtractVX(u8, u8),
    /// Subtract VY from VX and set to VX. VX = VY - VX. Affects the carry flag.
    SubtractVY(u8, u8),
    /// Set VX = VY >> 1. VF needs to be set to the bit that is shifted out.
    /// This instruction has different behaviour on CHIP-48 and SUPER-CHIP.
    ShiftRight(u8, u8),
    /// Set VX = VY << 1 VF needs to be set to the bit that is shifted out.
    /// This instruction has different behaviour on CHIP-48 and SUPER-CHIP.
    ShiftLeft(u8, u8),
    /// Jumps to the address and adds V0 offset.
    JumpWithOffset(u16),
    /// Generates a random number ANDed with the data and stores it in VX.
    GenerateRandomNumber(u8, u8),
    /// Skips the next instruction if VX is equal to data.
    SkipEqualVXData(u8, u8),
    /// Skip the next instruction if VX is not equal to data.
    SkipNotEqualVXData(u8, u8),
    /// Skips the next instruction if VX is equal to VY.
    SkipEqualVXVY(u8, u8),
    /// Skip the next instruction if VX is not equal to VY.
    SkipNotEqualVXVY(u8, u8),
    /// Sets the value of the VX instruction to the current value of the delay timer.
    SetVXToDelayTimer(u8),
    /// Sets the delay timer to the value in VX.
    SetDelayTimer(u8),
    /// Sets the sound timer to the value in VX.
    SetSoundTimer(u8),
    /// Adds the value of VX to the index register and sets the overflow flag.
    AddToIndex(u8),
    /// Sets the index register to the hexadecimal character in VX.
    FontCharacter(u8),
    /// Converts the number in VX to 3 digits.
    BinaryCodedDecimalConversion(u8),
    /// Stores the general purpose registers in memory at index register address.
    StoreMemory(u8),
    /// Loads the general purpose registers from memory at index register address.
    LoadMemory(u8),
    /// Blocks execution and waits for input. If a key is pressed it will be put in VX.
    GetKeyBlocking(u8),
    /// Skips one instruction if a key value stored in VX is pressed. Doesn't block execution.
    SkipIfKeyIsPressed(u8),
    /// Skips one instruction if a key value stored in VX is NOT pressed. Doesn't block execution.
    SkipIfKeyIsNotPressed(u8),
    /// Unknown instruction
    UnknownInstruction,
}

#[derive(Debug)]
pub struct Instruction {
    data: u16,
    processor_instruction: ProcessorInstruction,
}

impl Instruction {
    /// Creates a new instruction instance.
    pub(crate) fn new(data: [u8; 2]) -> Self {
        let data = ((data[0] as u16) << 8) | (data[1] as u16);
        Instruction {
            data,
            processor_instruction: Instruction::decode_instruction(data),
        }
    }
    /// raw returns the raw instruction data.
    pub fn raw(&self) -> u16 {
        self.data
    }

    /// Returns the processor instruction.
    pub fn processor_instruction(&self) -> ProcessorInstruction {
        self.processor_instruction
    }

    /// Decodes the raw instruction data into a processor instruction.
    fn decode_instruction(data: u16) -> ProcessorInstruction {
        let digit1 = Self::grab_zeroth_nibble(data);
        let digit2 = Self::grab_first_nibble(data);
        let digit3 = Self::grab_middle_nibble(data);
        let digit4 = Self::grab_last_nibble(data);
        match (digit1, digit2, digit3, digit4) {
            // Clear Display
            (0x0, 0x0, 0xE, 0x0) => ProcessorInstruction::ClearScreen,
            // Jump
            (0x1, _, _, _) => {
                // 1NNN
                ProcessorInstruction::Jump(Self::grab_inner_data(data))
            }
            // Set Register
            (0x6, _, _, _) => {
                // 6XNN
                ProcessorInstruction::SetRegister(
                    Self::grab_first_nibble(data),
                    Self::grab_last_byte(data),
                )
            }
            // Add value to register
            (0x7, _, _, _) => {
                // 7XNN
                ProcessorInstruction::AddValueToRegister(
                    Self::grab_first_nibble(data),
                    Self::grab_last_byte(data),
                )
            }
            // Set index register
            (0xA, _, _, _) => ProcessorInstruction::SetIndexRegister(Self::grab_inner_data(data)),
            // Draw on screen
            (0xD, _, _, _) => {
                // DXYN
                ProcessorInstruction::Draw(
                    Self::grab_first_nibble(data),
                    Self::grab_middle_nibble(data),
                    Self::grab_last_nibble(data),
                )
            }
            (0x0, 0x0, 0xE, 0xE) => ProcessorInstruction::Return,
            (0x2, _, _, _) => ProcessorInstruction::Call(Self::grab_inner_data(data)),
            (0x8, _, _, 0x0) => ProcessorInstruction::Set(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x1) => ProcessorInstruction::BinaryOr(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x2) => ProcessorInstruction::BinaryAnd(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x3) => ProcessorInstruction::BinaryXor(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x4) => ProcessorInstruction::Add(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x5) => ProcessorInstruction::SubtractVX(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x7) => ProcessorInstruction::SubtractVY(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0x6) => ProcessorInstruction::ShiftRight(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x8, _, _, 0xE) => ProcessorInstruction::ShiftLeft(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0xB, _, _, _) => ProcessorInstruction::JumpWithOffset(Self::grab_inner_data(data)),
            (0xC, _, _, _) => ProcessorInstruction::GenerateRandomNumber(
                Self::grab_first_nibble(data),
                Self::grab_last_byte(data),
            ),
            (0x3, _, _, _) => ProcessorInstruction::SkipEqualVXData(
                Self::grab_first_nibble(data),
                Self::grab_last_byte(data),
            ),
            (0x4, _, _, _) => ProcessorInstruction::SkipNotEqualVXData(
                Self::grab_first_nibble(data),
                Self::grab_last_byte(data),
            ),
            (0x5, _, _, 0x0) => ProcessorInstruction::SkipEqualVXVY(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0x9, _, _, 0x0) => ProcessorInstruction::SkipNotEqualVXVY(
                Self::grab_first_nibble(data),
                Self::grab_middle_nibble(data),
            ),
            (0xF, _, 0x0, 0x7) => {
                ProcessorInstruction::SetVXToDelayTimer(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x1, 0x5) => {
                ProcessorInstruction::SetDelayTimer(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x1, 0x8) => {
                ProcessorInstruction::SetSoundTimer(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x1, 0xE) => ProcessorInstruction::AddToIndex(Self::grab_first_nibble(data)),
            (0xF, _, 0x2, 0x9) => {
                ProcessorInstruction::FontCharacter(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x3, 0x3) => {
                ProcessorInstruction::BinaryCodedDecimalConversion(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x5, 0x5) => ProcessorInstruction::StoreMemory(Self::grab_first_nibble(data)),
            (0xF, _, 0x6, 0x5) => ProcessorInstruction::LoadMemory(Self::grab_first_nibble(data)),
            (0xE, _, 0x9, 0xE) => {
                ProcessorInstruction::SkipIfKeyIsPressed(Self::grab_first_nibble(data))
            }
            (0xE, _, 0xA, 0x1) => {
                ProcessorInstruction::SkipIfKeyIsNotPressed(Self::grab_first_nibble(data))
            }
            (0xF, _, 0x0, 0xA) => {
                ProcessorInstruction::GetKeyBlocking(Self::grab_first_nibble(data))
            }
            // Unknown instruction
            _ => ProcessorInstruction::UnknownInstruction,
        }
    }

    /// Grabs the inner data from the data, ignores the opcode.
    fn grab_inner_data(data: u16) -> u16 {
        data & 0x0FFF
    }

    /// Grabs the last byte from the data.
    fn grab_last_byte(data: u16) -> u8 {
        (data & 0x00FF) as u8
    }

    /// Grabs the zeroth nibble from the data.
    fn grab_zeroth_nibble(data: u16) -> u8 {
        ((data & 0xF000) >> 12) as u8
    }

    /// Grabs the first nibble from the data.
    fn grab_first_nibble(data: u16) -> u8 {
        ((data & 0x0F00) >> 8) as u8
    }

    /// Grabs the middle nibble from the data.
    fn grab_middle_nibble(data: u16) -> u8 {
        ((data & 0x00F0) >> 4) as u8
    }

    /// Grabs the last nibble from the data.
    fn grab_last_nibble(data: u16) -> u8 {
        (data & 0x000F) as u8
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "<Instruction: [{:02x}{:02x}]>",
            ((self.data & 0xFF00) >> 8u8) as u8,
            (self.data & 0x00FF) as u8
        ))
    }
}

impl PartialEq<u16> for Instruction {
    fn eq(&self, other: &u16) -> bool {
        self.data == *other
    }
}

impl LowerHex for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.data, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_raw() {
        let instruction = Instruction::new([0xffu8, 0xffu8]);

        assert_eq!(instruction.raw(), 0xffffu16)
    }

    #[test]
    fn test_instruction_trait_partial_eq() {
        let instruction = Instruction::new([0xffu8, 0xffu8]);

        assert_eq!(instruction, 0xffffu16)
    }

    #[test]
    fn test_instruction_clear_screen() {
        let instruction = Instruction::new([0x00, 0xE0]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::ClearScreen
        )
    }

    #[test]
    fn test_instruction_call() {
        let instruction = Instruction::new([0x2A, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::Call(0xABC)
        )
    }

    #[test]
    fn test_instruction_return() {
        let instruction = Instruction::new([0x00, 0xEE]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::Return
        )
    }

    #[test]
    fn test_instruction_skip_equal_vx_data() {
        let instruction = Instruction::new([0x3A, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipEqualVXData(0xA, 0xBC)
        )
    }

    #[test]
    fn test_instruction_skip_not_equal_vx_data() {
        let instruction = Instruction::new([0x4A, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipNotEqualVXData(0xA, 0xBC)
        )
    }

    #[test]
    fn test_instruction_skip_equal_vx_vy() {
        let instruction = Instruction::new([0x5A, 0xB0]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipEqualVXVY(0xA, 0xB)
        )
    }

    #[test]
    fn test_instruction_skip_not_equal_vx_vy() {
        let instruction = Instruction::new([0x9A, 0xB0]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipNotEqualVXVY(0xA, 0xB)
        )
    }

    #[test]
    fn test_instruction_set_register() {
        let instruction = Instruction::new([0x61, 0x40]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SetRegister(0x1, 0x40)
        )
    }

    #[test]
    fn test_instruction_add_to_register() {
        let instruction = Instruction::new([0x71, 0x40]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::AddValueToRegister(0x1, 0x40)
        )
    }

    #[test]
    fn test_instruction_set() {
        let instruction = Instruction::new([0x81, 0x40]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::Set(1, 4)
        )
    }

    #[test]
    fn test_instruction_binary_or() {
        let instruction = Instruction::new([0x81, 0xF1]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::BinaryOr(1, 0xF)
        )
    }

    #[test]
    fn test_instruction_binary_and() {
        let instruction = Instruction::new([0x81, 0xF2]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::BinaryAnd(1, 0xF)
        )
    }

    #[test]
    fn test_instruction_logical_xor() {
        let instruction = Instruction::new([0x81, 0xF3]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::BinaryXor(1, 0xF)
        )
    }

    #[test]
    fn test_instruction_logical_add() {
        let instruction = Instruction::new([0x81, 0xF4]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::Add(1, 0xF)
        )
    }

    #[test]
    fn test_instruction_logical_subtract_vx() {
        let instruction = Instruction::new([0x8E, 0xF5]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SubtractVX(0xE, 0xF)
        )
    }

    #[test]
    fn test_instruction_logical_subtract_vy() {
        let instruction = Instruction::new([0x8E, 0xF7]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SubtractVY(0xE, 0xF)
        )
    }

    #[test]
    fn test_instruction_shift_left() {
        let instruction = Instruction::new([0x81, 0x1E]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::ShiftLeft(1, 1)
        )
    }

    #[test]
    fn test_instruction_shift_right() {
        let instruction = Instruction::new([0x81, 0x26]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::ShiftRight(1, 2)
        )
    }

    #[test]
    fn test_instruction_set_index_register() {
        let instruction = Instruction::new([0xAA, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SetIndexRegister(0xABC)
        )
    }

    #[test]
    fn test_instruction_jump_with_offset() {
        let instruction = Instruction::new([0xBA, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::JumpWithOffset(0xABC)
        )
    }

    #[test]
    fn test_instruction_random() {
        let instruction = Instruction::new([0xCA, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::GenerateRandomNumber(0xA, 0xBC)
        )
    }

    #[test]
    fn test_instruction_display() {
        let instruction = Instruction::new([0xDA, 0xBC]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::Draw(0xA, 0xB, 0xC)
        )
    }

    #[test]
    fn test_instruction_set_vx_timer() {
        let instruction = Instruction::new([0xFA, 0x07]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SetVXToDelayTimer(0xA)
        )
    }

    #[test]
    fn test_instruction_set_delay_timer() {
        let instruction = Instruction::new([0xFA, 0x15]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SetDelayTimer(0xA)
        )
    }

    #[test]
    fn test_instruction_set_sound_timer() {
        let instruction = Instruction::new([0xFA, 0x18]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SetSoundTimer(0xA)
        )
    }

    #[test]
    fn test_instruction_add_to_index() {
        let instruction = Instruction::new([0xFA, 0x1E]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::AddToIndex(0xA)
        )
    }

    #[test]
    fn test_instruction_font_character() {
        let instruction = Instruction::new([0xFA, 0x29]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::FontCharacter(0xA)
        )
    }

    #[test]
    fn test_instruction_binary_coded_decimal() {
        let instruction = Instruction::new([0xFA, 0x33]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::BinaryCodedDecimalConversion(0xA)
        )
    }

    #[test]
    fn test_instruction_load_memory() {
        let instruction = Instruction::new([0xFA, 0x55]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::StoreMemory(0xA)
        )
    }

    #[test]
    fn test_instruction_store_memory() {
        let instruction = Instruction::new([0xFA, 0x65]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::LoadMemory(0xA)
        )
    }

    #[test]
    fn test_instruction_skip_if_key_pressed() {
        let instruction = Instruction::new([0xEF, 0x9E]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipIfKeyIsPressed(0xF)
        )
    }

    #[test]
    fn test_instruction_skip_if_key_not_pressed() {
        let instruction = Instruction::new([0xEF, 0xA1]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::SkipIfKeyIsNotPressed(0xF)
        )
    }

    #[test]
    fn test_instruction_get_key() {
        let instruction = Instruction::new([0xFE, 0x0A]);
        assert_eq!(
            instruction.processor_instruction,
            ProcessorInstruction::GetKeyBlocking(0xE)
        )
    }
}
