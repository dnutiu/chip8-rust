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
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Instruction: [{:02x}, {:02x}]",
            ((self.data & 0xFF00) >> 8u8) as u8,
            (self.data & 0x00FF) as u8
        ))
    }
}
