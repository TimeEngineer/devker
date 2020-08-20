//! # BlockType
//!
//! Help to determine which block is used for Huffman's coding.
//! - Raw = 0b00
//! - Fixed = 0b01
//! - Dynamic = 0b10

// Structures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Raw = 0b00,
    Fixed = 0b01,
    Dynamic = 0b10,
}
// Implementations.
impl BlockType {
    pub fn from(btype: u16) -> Self {
        match btype {
            0b00 => BlockType::Raw,
            0b01 => BlockType::Fixed,
            0b10 => BlockType::Dynamic,
            _ => unimplemented!(),
        }
    }
}
