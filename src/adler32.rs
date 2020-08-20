//! # Adler32

// Constants.
const BASE: u32 = 65521;
// Structures.
#[derive(Debug, Clone, Copy)]
pub struct Adler32(u32, u32);
// Implementations.
impl Adler32 {
    pub fn new() -> Self {
        Self(1, 0)
    }
    pub fn update(&mut self, buf: &[u8]) {
        for byte in buf {
            self.0 = (self.0 + *byte as u32) % BASE;
            self.1 = (self.1 + self.0) % BASE;
        }
    }
    pub fn checksum(&self) -> [u8; 4] {
        (self.1 << 16 | self.0).to_be_bytes()
    }
}
