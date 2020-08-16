//! # Huffman's Coding

// Imports.
use crate::bits::Bits;
use crate::code::Code;
use crate::prelude::BlockType;
// Structures.
#[derive(Debug)]
struct Writer {
    v_out: Vec<u8>,
    buf: u32,
    width: u8,
}
#[derive(Debug)]
struct HuffmanEncoder<'a> {
    literal: &'a mut [i32],
    distance: &'a mut [i32],
}
// Implementations.
impl Writer {
    fn new() -> Self {
        Self {
            v_out: Vec::new(),
            buf: 0,
            width: 0,
        }
    }
    fn write_bits(&mut self, bits: Bits) {
        self.buf |= (bits.data as u32) << self.width;
        self.width += bits.width;

        if self.width >= 16 {
            self.v_out.push(self.buf as u8);
            self.v_out.push((self.buf >> 8) as u8);
            self.width -= 16;
            self.buf >>= 16;
        }
    }
    fn finish(mut self) -> Vec<u8> {
        if self.width > 8 {
            self.v_out.push(self.buf as u8);
            self.v_out.push((self.buf >> 8) as u8);
        } else if self.width > 0 {
            self.v_out.push(self.buf as u8);
        }
        self.buf >>= 16;
        self.width = self.width.saturating_sub(16);
        self.v_out
    }
}
impl<'a> HuffmanEncoder<'a> {
    fn new(btype: BlockType, _v_in: &[Code], buf: &'a mut [i32; 0x10000]) -> Self {
        match btype {
            BlockType::Fixed => {
                let (literal, buf) = buf.split_at_mut(286);
                let (distance, _) = buf.split_at_mut(30);

                // Fixed Huffman Tree
                for i in 0..144 {
                    let (data, width) = (0b0_0011_0000 + i as u16, 8);
                    literal[i] = Bits { data, width }.reverse().as_i32();
                }
                for (i, j) in (144..256).enumerate() {
                    let (data, width) = (0b1_1001_0000 + i as u16, 9);
                    literal[j] = Bits { data, width }.reverse().as_i32();
                }
                for (i, j) in (256..280).enumerate() {
                    let (data, width) = (0b0_0000_0000 + i as u16, 7);
                    literal[j] = Bits { data, width }.reverse().as_i32();
                }
                for (i, j) in (280..286).enumerate() {
                    let (data, width) = (0b0_1100_0000 + i as u16, 8);
                    literal[j] = Bits { data, width }.reverse().as_i32();
                }
                for i in 0..30 {
                    let (width, data) = (5, i as u16);
                    distance[i] = Bits { data, width }.reverse().as_i32();
                }

                Self { literal, distance }
            }
            _ => unimplemented!(),
        }
    }
    fn encode(&self, writer: &mut Writer, code: Code) {
        let lcode = self.literal[code.literal_code() as usize];
        let bits = Bits::from(lcode);
        writer.write_bits(bits);

        if let Some((width, data)) = code.extra_length() {
            writer.write_bits(Bits { data, width });
        }
        if let Some((code, width, data)) = code.distance_code() {
            let dcode = self.distance[code as usize];
            let bits = Bits::from(dcode);
            writer.write_bits(bits);
            writer.write_bits(Bits { data, width });
        }
    }
}
// Main functions.
pub fn huffman_encode(v_in: &[Code], btype: BlockType, buf: &mut [i32; 0x10000]) -> Vec<u8> {
    // Variable Initialization.
    let mut writer = Writer::new();

    // Algorithms.
    let data = btype as u16;
    writer.write_bits(Bits { data: 1, width: 1 });
    writer.write_bits(Bits { data, width: 2 });
    let encoder = HuffmanEncoder::new(btype, v_in, buf);
    for code in v_in {
        encoder.encode(&mut writer, *code);
    }
    writer.finish()
}
