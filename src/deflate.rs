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
