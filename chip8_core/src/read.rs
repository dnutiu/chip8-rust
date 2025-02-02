#[cfg(feature = "std")]
use std::fs::File;

#[cfg(feature = "std")]
use std::io::Read;

pub trait Reader {
    /// Reads the bytes into the buffer and returns the amount read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, anyhow::Error>;
}

#[cfg(feature = "std")]
pub struct StdFileReader {
    file: File,
}

#[cfg(feature = "std")]
impl StdFileReader {
    pub fn new(file: File) -> Self {
        StdFileReader { file }
    }
}

#[cfg(feature = "std")]
impl Reader for StdFileReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, anyhow::Error> {
        let amount = self.file.read(buf)?;
        Ok(amount)
    }
}
