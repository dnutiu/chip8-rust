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

#[derive(Debug, Clone, Copy)]
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
        let data = (data[0] as u16) << 8u8 | (data[1] as u16);
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
        let digit4 = Self::grab_first_nibble(data);
        match (digit1, digit2, digit3, digit4) {
            // Clear Display
            (0, 0, 0xE, 0) => ProcessorInstruction::ClearScreen,
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
            /*
            00EE and 2NNN:

            2NNN calls the subroutine at memory location NNN. In other words, just like 1NNN,
            you should set PC to NNN. However, the difference between a jump and a call is that
            this instruction should first push the current PC to the stack, so the subroutine can return later.

            Returning from a subroutine is done with 00EE, and it does this by removing
            (“popping”) the last address from the stack and setting the PC to it.
             */
            (0x0, 0x0, 0xE, 0xE) => ProcessorInstruction::Return,
            (0x2, _, _, _) => ProcessorInstruction::Call(Self::grab_inner_data(data)),
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
    fn test_instruction_partial_eq() {
        let instruction = Instruction::new([0xffu8, 0xffu8]);

        assert_eq!(instruction, 0xffffu16)
    }
}
