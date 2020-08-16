//! # Bits

// Structures.
#[derive(Debug, Clone, Copy)]
pub struct Bits {
    pub data: u16,
    pub width: u8,
}
// Implementations.
impl Bits {
    pub fn from(code: i32) -> Self {
        let width = (code >> 16) as u8;
        let data = (code & 0xFFFF) as u16;
        Self { data, width }
    }
    pub fn reverse(&self) -> Self {
        let width = self.width;
        let data = self.data.reverse_bits() >> (16 - width);
        Self { data, width }
    }
    pub fn as_i32(&self) -> i32 {
        let (b0, b1) = ((self.data >> 8) as u8, (self.data & 0xFF) as u8);
        i32::from_be_bytes([0, self.width, b0, b1])
    }
}
