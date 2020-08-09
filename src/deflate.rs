//! # Deflate/Inflate
//!
//! ## Examples
//!
//! ```
//! use devker::huffman::BlockType;
//! use devker::deflate::{deflate, inflate};
//! 
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! // Encode
//! let encoded = deflate(&v_in, BlockType::Fixed);
//! // Decode
//! let decoded = inflate(&encoded).unwrap();
//! assert_eq!(v_in, decoded);
//! ```

// Imports.
use crate::code::Code;
use crate::huffman::{huffman_decode, huffman_encode, BlockType};
use crate::lzss::{lzss_decode, lzss_encode};
// Main functions.
pub fn deflate(v_in: &[u8], btype: BlockType) -> Vec<u8> {
    // Algorithm.
    let mut encoded = lzss_encode(v_in);
    encoded.push(Code::EndOfBlock);
    huffman_encode(&encoded, btype)
}

pub fn inflate(v_in: &[u8]) -> Result<Vec<u8>, String> {
    // Algorithm.
    let decoded = huffman_decode(&v_in)?;
    lzss_decode(&decoded)
}
