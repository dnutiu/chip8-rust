use std::fmt;
use std::fmt::{Display, Formatter, LowerHex};

#[derive(Debug)]
pub struct Instruction {
    data: u16,
}

impl Instruction {
    /// Creates a new instruction instance.
    pub(crate) fn new(data: [u8; 2]) -> Self {
        Instruction {
            data: (data[0] as u16) << 8u8 | (data[1] as u16),
        }
    }

    /// raw returns the raw instruction data.
    pub fn raw(&self) -> u16 {
        self.data
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "Instruction: [{:02x}, {:02x}]",
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
