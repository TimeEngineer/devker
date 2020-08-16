//! # Deflate/Inflate
//!
//! ## Examples
//!
//! ### Easy to use.
//! ```
//! use devker::prelude::{deflate, inflate, BlockType, Cache};
//!
//! let mut cache = Cache::new();
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! // Encode.
//! let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
//! // Decode.
//! let decoded = inflate(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//! ```
//!
//! ### Reusable cache.
//! ```
//! use devker::prelude::{deflate, inflate, BlockType, Cache};
//!
//! let mut cache = Cache::new();
//!
//! // First try.
//!
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
//! let decoded = inflate(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//!
//! // Another try.
//!
//! let v = String::from("The cache can be reused !");
//! let v_in = v.into_bytes();
//!
//! let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
//! let decoded = inflate(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//! ```

// Imports.
use crate::bits::Bits;
use crate::code::Code;
use crate::code::{DISTANCE_TABLE, END_OF_BLOCK, LENGTH_TABLE};
use crate::huffman::huffman_encode;
use crate::lzss::{extend, lzss_encode};
use crate::prelude::{BlockType, Cache};
// Constants.
const ERROR_BUFFER: &str = "Buffer overflow.";
const ERROR_COMPLEMENT: &str = "LEN is not the one's complement of NLEN.";
const ERROR_LENGTH: &str = "Invalid length.";
const ERROR_PREVIOUS: &str = "No previous value.";
const ERROR_RESERVED: &str = "Reserved btype.";
const ERROR_VALUE: &str = "Invalid value decoded.";
const ERROR_WIDTH: &str = "Invalid width decoded";
const ERROR_WIDTHES: &str = "Invalid code lengths.";
const WIDTH_CODE_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
// Structures.
type IterU8 = dyn Iterator<Item = u8>;
#[derive(Debug)]
struct Reader<'a> {
    v_in: &'a [u8],
    last_read: u32,
    offset: u8,
    last_error: Option<&'a str>,
}
#[derive(Debug)]
struct HuffmanDecoder<'a> {
    literal: &'a mut [i32],
    distance: &'a mut [i32],
    eob_width: u8,
    max_lwidth: u8,
    max_dwidth: u8,
}
// Implementations.
impl<'a> Reader<'a> {
    fn new(v_in: &'a [u8]) -> Self {
        Self {
            v_in,
            last_read: 0,
            offset: 32,
            last_error: None,
        }
    }
    fn read(&mut self) -> u8 {
        if self.v_in.is_empty() {
            self.last_error = Some(ERROR_BUFFER);
            return 0;
        }
        let next = self.v_in[0];
        self.v_in = &self.v_in[1..];
        next
    }
    fn read_u16(&mut self) -> u16 {
        if self.v_in.len() < 2 {
            self.last_error = Some(ERROR_BUFFER);
            return 0;
        }
        let mut array = [0; 2];
        (&mut array).copy_from_slice(&self.v_in[0..2]);
        let next = u16::from_le_bytes(array);
        self.v_in = &self.v_in[2..];
        next
    }
    fn read_bits(&mut self, width: u8) -> u16 {
        let bits = self.peek_bits(width);
        self.skip_bits(width);
        bits
    }
    fn peek_bits(&mut self, width: u8) -> u16 {
        while self.offset > 32 - width {
            let next = self.read() as u32;
            self.offset -= 8;
            self.last_read >>= 8;
            self.last_read |= next << 24;
        }
        let mut bits = self.last_read.wrapping_shr(self.offset as u32) as u16;
        let mask = (1 << width) - 1;
        bits &= mask;
        bits
    }
    fn skip_bits(&mut self, width: u8) {
        self.offset += width;
    }
    fn reset(&mut self) {
        self.offset = 32;
    }
    fn get_code(&mut self, decoder: &[i32], max_width: u8) -> u16 {
        let code = self.peek_bits(max_width);
        let bits = Bits::from(decoder[code as usize]);
        if bits.width > max_width {
            self.last_error = Some(ERROR_WIDTH);
            return 0;
        }
        self.skip_bits(bits.width);
        bits.data
    }
    fn check_last_error(&self) -> Result<(), String> {
        if let Some(e) = self.last_error {
            return Err(e.into());
        }
        Ok(())
    }
    fn load_widthes(&mut self, code: u16, last: Option<u8>) -> Result<Box<IterU8>, String> {
        self.check_last_error()?;
        Ok(match code {
            0..=15 => Box::new(std::iter::once(code as u8)),
            16 => {
                let count = self.read_bits(2) + 3;
                let last = match last {
                    Some(x) => x,
                    None => return Err(ERROR_PREVIOUS.into()),
                };
                Box::new(std::iter::repeat(last).take(count as usize))
            }
            17 => {
                let zeros = self.read_bits(3) + 3;
                Box::new(std::iter::repeat(0).take(zeros as usize))
            }
            18 => {
                let zeros = self.read_bits(7) + 11;
                Box::new(std::iter::repeat(0).take(zeros as usize))
            }
            _ => return Err(ERROR_WIDTHES.into()),
        })
    }
    fn fill(&mut self, v_out: &mut Vec<u8>, len: usize) -> Result<(), String> {
        self.check_last_error()?;
        if self.v_in.len() < len {
            return Err(ERROR_BUFFER.into());
        }
        v_out.reserve(len);
        let mut array = vec![0; len];
        (&mut array).copy_from_slice(&self.v_in[..len]);
        v_out.extend(array);
        Ok(())
    }
}
impl<'a> HuffmanDecoder<'a> {
    fn new(
        btype: BlockType,
        reader: &mut Reader,
        buf: &'a mut [i32; 0x10000],
    ) -> Result<Self, String> {
        Ok(match btype {
            BlockType::Fixed => {
                let max_lwidth = 9;
                let max_dwidth = 5;
                let (literal, buf) = buf.split_at_mut(1 << max_lwidth);
                let (distance, _) = buf.split_at_mut(1 << max_dwidth);

                // Fixed Huffman Tree
                for i in 0..144 {
                    let (data, width) = (0b0_0011_0000 + i as u16, 8);
                    Self::set_mapping(literal, i, data, width, max_lwidth);
                }
                for (i, code) in (144..256).enumerate() {
                    let (data, width) = (0b1_1001_0000 + i as u16, 9);
                    Self::set_mapping(literal, code, data, width, max_lwidth);
                }
                for (i, code) in (256..280).enumerate() {
                    let (data, width) = (0b0_0000_0000 + i as u16, 7);
                    Self::set_mapping(literal, code, data, width, max_lwidth);
                }
                for (i, code) in (280..288).enumerate() {
                    let (data, width) = (0b0_1100_0000 + i as u16, 8);
                    Self::set_mapping(literal, code, data, width, max_lwidth);
                }
                for i in 0..30 {
                    let (data, width) = (i as u16, 5);
                    Self::set_mapping(distance, data, data, width, max_dwidth);
                }

                Self {
                    literal,
                    distance,
                    eob_width: 7,
                    max_lwidth,
                    max_dwidth,
                }
            }
            BlockType::Dynamic => {
                let lcount = reader.read_bits(5) as usize + 257;
                let dcount = reader.read_bits(5) as usize + 1;
                let wcount = reader.read_bits(4) as usize + 4;

                // Width decoder.
                let mut width_code_widthes = [0; 19];
                for i in WIDTH_CODE_ORDER.iter().take(wcount) {
                    width_code_widthes[*i] = reader.read_bits(3) as u8;
                }
                let (width_decoder, buf, max_wwidth, _) =
                    Self::from_widthes(buf, &width_code_widthes);

                // Literal.
                let mut literal_code_widthes = Vec::with_capacity(lcount);
                while literal_code_widthes.len() < lcount {
                    let code = reader.get_code(width_decoder, max_wwidth);
                    let last = literal_code_widthes.last().copied();
                    literal_code_widthes.extend(reader.load_widthes(code, last)?);
                }

                // Distance.
                let mut distance_code_widthes =
                    literal_code_widthes.drain(lcount..).collect::<Vec<_>>();
                distance_code_widthes.reserve(dcount);
                while distance_code_widthes.len() < dcount {
                    let code = reader.get_code(width_decoder, max_wwidth);
                    let last = distance_code_widthes
                        .last()
                        .copied()
                        .or_else(|| literal_code_widthes.last().copied());
                    distance_code_widthes.extend(reader.load_widthes(code, last)?);
                }
                if distance_code_widthes.len() > dcount {
                    return Err(ERROR_LENGTH.into());
                }
                let (literal, buf, max_lwidth, eob_width) =
                    Self::from_widthes(buf, &literal_code_widthes);
                let (distance, _, max_dwidth, _) = Self::from_widthes(buf, &distance_code_widthes);
                Self {
                    literal,
                    distance,
                    eob_width,
                    max_lwidth,
                    max_dwidth,
                }
            }
            _ => unimplemented!(),
        })
    }
    fn set_mapping(decoder: &mut [i32], code: u16, data: u16, width: u8, max_width: u8) {
        let bits = Bits { data, width }.reverse();
        for padding in 0..(1 << (max_width - width)) {
            decoder[padding << width | bits.data as usize] = Bits { data: code, width }.as_i32();
        }
    }
    fn from_widthes(buf: &'a mut [i32], widthes: &[u8]) -> (&'a mut [i32], &'a mut [i32], u8, u8) {
        let max_width = *widthes.iter().max().unwrap_or(&0);
        let (decoder, buf) = buf.split_at_mut(1 << max_width);
        let eob_width = Self::restore_canonical_huffman_codes(decoder, &widthes, max_width as u8);
        (decoder, buf, max_width, eob_width)
    }
    fn restore_canonical_huffman_codes(width: &mut [i32], widthes: &[u8], max_width: u8) -> u8 {
        let mut codes = widthes
            .iter()
            .enumerate()
            .filter(|(_, code_width)| **code_width > 0)
            .map(|(code, code_width)| (code as u16, *code_width))
            .collect::<Vec<_>>();
        codes.sort_by_key(|x| x.1);

        let mut code = 0;
        let mut prev_width = 0;
        let mut eob_width = 0;
        for (c, w) in codes {
            code <<= w - prev_width;
            Self::set_mapping(width, c, code, w, max_width);
            if c == END_OF_BLOCK {
                eob_width = w;
            }
            code += 1;
            prev_width = w;
        }
        eob_width
    }
    fn decode(&self, reader: &mut Reader) -> Result<Code, String> {
        let code = reader.peek_bits(self.eob_width);
        let mut bits = Bits::from(self.literal[code as usize]);
        if bits.width > self.eob_width {
            let code = reader.peek_bits(self.max_lwidth);
            bits = Bits::from(self.literal[code as usize]);
            if bits.width > self.max_lwidth {
                return Err(ERROR_WIDTH.into());
            }
        }
        reader.skip_bits(bits.width);
        reader.check_last_error()?;

        Ok(match bits.data {
            0..=255 => Code::Literal(bits.data as u8),
            256 => Code::EndOfBlock,
            length_code @ 257..=285 => {
                let (code_base_length, width_length) = LENGTH_TABLE[length_code as usize - 257];
                let bits_length = reader.read_bits(width_length);

                let code = reader.get_code(self.distance, self.max_dwidth);

                let (code_base_distance, width_distance) = DISTANCE_TABLE[code as usize];
                let bits_distance = reader.read_bits(width_distance);

                Code::Pointer {
                    length: code_base_length + bits_length as u8,
                    distance: code_base_distance + bits_distance,
                }
            }
            _ => return Err(ERROR_VALUE.into()),
        })
    }
}
// Main functions.
pub fn deflate(v_in: &[u8], btype: BlockType, cache: &mut Cache) -> Vec<u8> {
    // Variable Initialization.
    let buf = cache.inner_mut();
    // Algorithm.
    let mut encoded = lzss_encode(v_in, buf);
    encoded.push(Code::EndOfBlock);
    huffman_encode(&encoded, btype, buf)
}

pub fn inflate(v_in: &[u8], cache: &mut Cache) -> Result<Vec<u8>, String> {
    // Variable Initialization.
    let buf = cache.inner_mut();
    let mut reader = Reader::new(v_in);
    let mut bfinal = 0;
    let mut v_out = Vec::new();

    // Algorithms.
    while bfinal == 0 {
        bfinal = reader.read_bits(1);
        let btype = reader.read_bits(2);
        reader.check_last_error()?;
        match btype {
            0b11 => return Err(ERROR_RESERVED.into()),
            0b00 => {
                reader.reset();
                let len = reader.read_u16();
                let nlen = reader.read_u16();
                if !len != nlen {
                    return Err(ERROR_COMPLEMENT.into());
                }
                reader.fill(&mut v_out, len as usize)?;
            }
            btype => {
                let decoder = HuffmanDecoder::new(BlockType::from(btype), &mut reader, buf)?;
                loop {
                    let x = decoder.decode(&mut reader)?;
                    match x {
                        Code::EndOfBlock => break,
                        Code::Literal(a) => v_out.push(a),
                        Code::Pointer {
                            distance: d,
                            length: l,
                        } => extend(&mut v_out, d as usize, l as usize + 3)?,
                    }
                }
            }
        }
    }
    Ok(v_out)
}
