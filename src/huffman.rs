// Imports.
use crate::code::Code;
use crate::code::{DISTANCE_TABLE, END_OF_BLOCK, FIXED_LITERAL_BIT_CODE_TABLE, LENGTH_TABLE};
use std::collections::BinaryHeap;
// Constants.
pub const WIDTH_CODE_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
pub const ERROR_BUFFER: &str = "Buffer overflow.";
pub const ERROR_VALUE: &str = "Invalid value decoded.";
pub const ERROR_WIDTH: &str = "Invalid width decoded";
pub const ERROR_RESERVED: &str = "Reserved btype.";
pub const ERROR_PREVIOUS: &str = "No previous value.";
pub const ERROR_WIDTHES: &str = "Invalid code lengths.";
pub const ERROR_LENGTH: &str = "Invalid length.";
pub const ERROR_COMPLEMENT: &str = "LEN is not the one's complement of NLEN.";
// Structures.
pub type IterU8 = dyn Iterator<Item = u8>;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Raw = 0b00,
    Fixed = 0b01,
    Dynamic = 0b10,
}
#[derive(Debug, Clone, Copy)]
pub struct Bits {
    pub data: u16,
    pub width: u8,
}
#[derive(Debug, Clone)]
pub struct Node {
    pub symbols: Vec<u16>,
    pub weight: usize,
}
#[derive(Debug)]
pub struct Writer {
    v_out: Vec<u8>,
    buf: u32,
    width: u8,
}
#[derive(Debug)]
pub struct Reader<'a> {
    v_in: &'a [u8],
    pos: usize,
    last_read: u32,
    offset: u8,
}
#[derive(Debug)]
pub struct Encoder(Vec<Bits>);
#[derive(Debug)]
pub struct Decoder {
    table: Vec<u16>,
    eob_literal: Option<u16>,
    eob_width: u8,
    max_width: u8,
}
#[derive(Debug)]
pub struct HuffmanEncoder {
    literal: Encoder,
    distance: Encoder,
}
#[derive(Debug)]
pub struct HuffmanDecoder {
    literal: Decoder,
    distance: Decoder,
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
impl Bits {
    pub fn reverse(&self) -> Self {
        let data = self.data.reverse_bits() >> (16 - self.width);
        Bits {
            data: data,
            width: self.width,
        }
    }
}
impl Node {
    pub fn empty() -> Self {
        Node {
            symbols: vec![],
            weight: 0,
        }
    }
    pub fn single(symbol: u16, weight: usize) -> Self {
        Node {
            symbols: vec![symbol],
            weight,
        }
    }
    pub fn merge(&mut self, other: Self) {
        self.weight += other.weight;
        self.symbols.extend(other.symbols);
    }
}
impl Writer {
    pub fn new() -> Self {
        Self {
            v_out: Vec::new(),
            buf: 0,
            width: 0,
        }
    }
    pub fn write_bits(&mut self, bits: Bits) {
        self.buf |= (bits.data as u32) << self.width;
        self.width += bits.width;

        if self.width >= 16 {
            self.v_out.push(self.buf as u8);
            self.v_out.push((self.buf >> 8) as u8);
            self.width -= 16;
            self.buf >>= 16;
        }
    }
    pub fn finish(mut self) -> Vec<u8> {
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
impl<'a> Reader<'a> {
    pub fn new(v_in: &'a [u8]) -> Self {
        Self {
            v_in: v_in,
            pos: 0,
            last_read: 0,
            offset: 32,
        }
    }
    pub fn read_u8(&mut self) -> Result<u8, String> {
        if self.pos >= self.v_in.len() {
            return Err(ERROR_BUFFER.into());
        }
        let next = self.v_in[self.pos];
        self.pos += 1;
        Ok(next)
    }
    pub fn read_u16(&mut self) -> Result<u16, String> {
        if self.pos + 1 >= self.v_in.len() {
            return Err(ERROR_BUFFER.into());
        }
        let next = (self.v_in[self.pos] as u16) << 8 | (self.v_in[self.pos + 1] as u16);
        self.pos += 2;
        Ok(next)
    }
    pub fn read_bits(&mut self, width: u8) -> Result<u16, String> {
        let bits = self.peek_bits(width)?;
        self.offset += width;
        Ok(bits)
    }
    pub fn peek_bits(&mut self, width: u8) -> Result<u16, String> {
        while self.offset + width > 32 {
            let next = self.read_u8()? as u32;
            self.offset -= 8;
            self.last_read >>= 8;
            self.last_read |= next << 24;
        }
        let mut bits = self.last_read.wrapping_shr(self.offset as u32) as u16;
        let mask = (1 << width) - 1;
        bits &= mask;
        Ok(bits)
    }
    pub fn reset(&mut self) {
        self.offset = 32;
    }
    pub fn load_widthes(&mut self, code: u16, last: Option<u8>) -> Result<Box<IterU8>, String> {
        Ok(match code {
            0..=15 => Box::new(std::iter::once(code as u8)),
            16 => {
                let count = self.read_bits(2)? + 3;
                let last = match last {
                    Some(x) => x,
                    None => return Err(ERROR_PREVIOUS.into()),
                };
                Box::new(std::iter::repeat(last).take(count as usize))
            }
            17 => {
                let zeros = self.read_bits(3)? + 3;
                Box::new(std::iter::repeat(0).take(zeros as usize))
            }
            18 => {
                let zeros = self.read_bits(7)? + 11;
                Box::new(std::iter::repeat(0).take(zeros as usize))
            }
            _ => return Err(ERROR_WIDTHES.into()),
        })
    }
}
pub trait Builder {
    fn set_mapping(&mut self, literal: u16, bits: Bits);
    fn restore_canonical_huffman_codes(&mut self, widthes: &[u8]) {
        let mut codes = widthes
            .iter()
            .enumerate()
            .filter(|(_, code_width)| **code_width > 0)
            .map(|(code, code_width)| (code as u16, *code_width))
            .collect::<Vec<_>>();
        codes.sort_by_key(|x| x.1);

        let mut code = 0;
        let mut prev_width = 0;
        for (c, w) in codes {
            code <<= w - prev_width;
            self.set_mapping(
                c,
                Bits {
                    data: code,
                    width: w,
                },
            );
            code += 1;
            prev_width = w;
        }
    }
}
impl Builder for Encoder {
    fn set_mapping(&mut self, literal: u16, bits: Bits) {
        self.0[literal as usize] = bits.reverse();
    }
}
impl Encoder {
    pub fn new(n: usize) -> Self {
        Self(vec![Bits { data: 0, width: 0 }; n])
    }
    pub fn from_widthes(widthes: &[u8]) -> Self {
        let code_count = widthes
            .iter()
            .enumerate()
            .filter(|e| *e.1 > 0)
            .last()
            .map_or(0, |e| e.0)
            + 1;
        let mut encoder = Self::new(code_count);
        encoder.restore_canonical_huffman_codes(widthes);
        encoder
    }
    pub fn from_frequencies(frequencies: &[usize], max_width: u8) -> Self {
        let max_width = std::cmp::min(max_width, calc_optimal_max_width(frequencies));
        let code_widthes = calc(max_width, frequencies);
        Self::from_widthes(&code_widthes)
    }
    pub fn lookup(&self, literal: u16) -> Bits {
        self.0[literal as usize]
    }
    pub fn max_code(&self) -> Option<u16> {
        self.0
            .iter()
            .rev()
            .position(|x| x.width > 0)
            .map(|trailing_zeros| (self.0.len() - 1 - trailing_zeros) as u16)
    }
    pub fn encode(&self, writer: &mut Writer, literal: u16) {
        let bits = self.lookup(literal);
        writer.write_bits(bits);
    }
}
impl Builder for Decoder {
    fn set_mapping(&mut self, literal: u16, bits: Bits) {
        if Some(literal) == self.eob_literal {
            self.eob_width = bits.width;
        }
        let value = (literal << 5) | bits.width as u16;
        let reverse = bits.reverse().data;
        for padding in 0..(1 << (self.max_width - bits.width)) {
            let i = ((padding << bits.width) | reverse) as usize;
            self.table[i] = value;
        }
    }
}
impl Decoder {
    pub fn new(max_width: u8, eob_literal: Option<u16>) -> Self {
        Self {
            table: vec![16; 1 << max_width],
            eob_literal: eob_literal,
            eob_width: max_width,
            max_width: max_width,
        }
    }
    pub fn from_widthes(widthes: &[u8], eob_literal: Option<u16>) -> Self {
        let mut decoder = Self::new(*widthes.iter().max().unwrap_or(&0), eob_literal);
        decoder.restore_canonical_huffman_codes(widthes);
        decoder
    }
    pub fn decode(&self, reader: &mut Reader) -> Result<u16, String> {
        let code = reader.peek_bits(self.eob_width)?;
        let mut value = self.table[code as usize];
        let mut width = (value & 0b1_1111) as u8;
        if width > self.eob_width {
            let code = reader.peek_bits(self.max_width)?;
            value = self.table[code as usize];
            width = (value & 0b1_1111) as u8;
            if width > self.max_width {
                return Err(ERROR_WIDTH.into());
            }
        }
        reader.offset += width as u8;
        Ok(value >> 5)
    }
}
impl HuffmanEncoder {
    pub fn new(btype: BlockType, v_in: &[Code]) -> Self {
        match btype {
            BlockType::Fixed => {
                let mut literal = Encoder::new(288);
                let mut distance = Encoder::new(30);

                for (literals, width, code_base) in &FIXED_LITERAL_BIT_CODE_TABLE {
                    for (i, lit) in literals.clone().enumerate() {
                        literal.set_mapping(
                            lit,
                            Bits {
                                data: *code_base + i as u16,
                                width: *width,
                            },
                        );
                    }
                }
                for i in 0..30 {
                    distance.set_mapping(i, Bits { data: i, width: 5 });
                }
                Self { literal, distance }
            }
            BlockType::Dynamic => {
                let mut literal_counts = [0; 286];
                let mut distance_counts = [0; 30];

                for code in v_in {
                    literal_counts[code.literal_code() as usize] += 1;
                    if let Some((d, _, _)) = code.distance_code() {
                        distance_counts[d as usize] += 1;
                    }
                }
                let literal = Encoder::from_frequencies(&literal_counts, 15);
                let distance = Encoder::from_frequencies(&distance_counts, 15);
                Self { literal, distance }
            }
            _ => unimplemented!(),
        }
    }
    pub fn build_width_codes(&self, lcount: u16, dcount: u16) -> Vec<(u8, u8, u8)> {
        let mut run_lens: Vec<(u8, usize)> = Vec::new();
        for (e, size) in &[(&self.literal, lcount), (&self.distance, dcount)] {
            for (i, c) in (0..*size).map(|x| e.lookup(x as u16).width).enumerate() {
                if i > 0 && run_lens.last().map_or(false, |s| s.0 == c) {
                    run_lens.last_mut().unwrap().1 += 1;
                } else {
                    run_lens.push((c, 1))
                }
            }
        }

        let mut codes: Vec<(u8, u8, u8)> = Vec::new();
        for r in run_lens {
            if r.0 == 0 {
                let mut c = r.1;
                while c >= 11 {
                    let n = std::cmp::min(138, c) as u8;
                    codes.push((18, 7, n - 11));
                    c -= n as usize;
                }
                if c >= 3 {
                    codes.push((17, 3, c as u8 - 3));
                    c = 0;
                }
                for _ in 0..c {
                    codes.push((0, 0, 0));
                }
            } else {
                codes.push((r.0, 0, 0));
                let mut c = r.1 - 1;
                while c >= 3 {
                    let n = std::cmp::min(6, c) as u8;
                    codes.push((16, 2, n - 3));
                    c -= n as usize;
                }
                for _ in 0..c {
                    codes.push((r.0, 0, 0));
                }
            }
        }
        codes
    }
    pub fn save(&self, writer: &mut Writer) {
        let lcount = std::cmp::max(257, self.literal.max_code().unwrap_or(0) + 1);
        let dcount = std::cmp::max(1, self.distance.max_code().unwrap_or(0) + 1);
        let codes = self.build_width_codes(lcount, dcount);

        let mut code_counts = [0; 19];
        for x in &codes {
            code_counts[x.0 as usize] += 1;
        }
        let width_encoder = Encoder::from_frequencies(&code_counts, 7);

        let width = WIDTH_CODE_ORDER
            .iter()
            .rev()
            .position(|&i| code_counts[i] != 0 && width_encoder.lookup(i as u16).width > 0)
            .map_or(0, |trailing_zeros| 19 - trailing_zeros);
        let wcount = std::cmp::max(4, width) as u16;
        writer.write_bits(Bits {
            data: lcount - 257,
            width: 5,
        });
        writer.write_bits(Bits {
            data: dcount - 1,
            width: 5,
        });
        writer.write_bits(Bits {
            data: wcount - 4,
            width: 4,
        });
        for i in WIDTH_CODE_ORDER.iter().take(wcount as usize) {
            let width = match code_counts[*i] {
                0 => 0,
                _ => width_encoder.lookup(*i as u16).width as u16,
            };
            writer.write_bits(Bits {
                data: width,
                width: 3,
            });
        }
        for (code, bits, extra) in codes {
            width_encoder.encode(writer, code as u16);
            if bits > 0 {
                writer.write_bits(Bits {
                    data: extra as u16,
                    width: bits,
                });
            }
        }
    }
    pub fn encode(&self, writer: &mut Writer, code: Code) {
        self.literal.encode(writer, code.literal_code());
        if let Some((width, data)) = code.extra_length() {
            writer.write_bits(Bits { data, width });
        }
        if let Some((code, width, data)) = code.distance_code() {
            self.distance.encode(writer, code);
            writer.write_bits(Bits { data, width });
        }
    }
}
impl HuffmanDecoder {
    pub fn new(btype: BlockType, reader: &mut Reader) -> Result<Self, String> {
        Ok(match btype {
            BlockType::Fixed => {
                let mut literal = Decoder::new(9, Some(END_OF_BLOCK));
                let mut distance = Decoder::new(5, None);

                for (literals, width, code_base) in &FIXED_LITERAL_BIT_CODE_TABLE {
                    for (i, lit) in literals.clone().enumerate() {
                        literal.set_mapping(
                            lit,
                            Bits {
                                data: *code_base + i as u16,
                                width: *width,
                            },
                        );
                    }
                }
                for i in 0..30 {
                    distance.set_mapping(i, Bits { data: i, width: 5 });
                }
                Self { literal, distance }
            }
            BlockType::Dynamic => {
                let lcount = reader.read_bits(5)? as usize + 257;
                let dcount = reader.read_bits(5)? as usize + 1;
                let wcount = reader.read_bits(4)? as usize + 4;

                // Width decoder.
                let mut width_code_widthes = [0; 19];
                for i in WIDTH_CODE_ORDER.iter().take(wcount) {
                    width_code_widthes[*i] = reader.read_bits(3)? as u8;
                }
                let width_decoder = Decoder::from_widthes(&width_code_widthes, None);

                // Literal.
                let mut literal_code_widthes = Vec::with_capacity(lcount);
                while literal_code_widthes.len() < lcount {
                    let code = width_decoder.decode(reader)?;
                    let last = literal_code_widthes.last().copied();
                    literal_code_widthes.extend(reader.load_widthes(code, last)?);
                }

                // Distance.
                let mut distance_code_widthes =
                    literal_code_widthes.drain(lcount..).collect::<Vec<_>>();
                distance_code_widthes.reserve(dcount);
                while distance_code_widthes.len() < dcount {
                    let code = width_decoder.decode(reader)?;
                    let last = distance_code_widthes
                        .last()
                        .copied()
                        .or_else(|| literal_code_widthes.last().copied());
                    distance_code_widthes.extend(reader.load_widthes(code, last)?);
                }
                if distance_code_widthes.len() > dcount {
                    return Err(ERROR_LENGTH.into());
                }
                let literal = Decoder::from_widthes(&literal_code_widthes, Some(END_OF_BLOCK));
                let distance = Decoder::from_widthes(&distance_code_widthes, None);
                Self { literal, distance }
            }
            _ => unimplemented!(),
        })
    }
    pub fn decode(&self, reader: &mut Reader) -> Result<Code, String> {
        let decoded = self.literal.decode(reader)?;
        Ok(match decoded {
            0..=255 => Code::Literal(decoded as u8),
            256 => Code::EndOfBlock,
            286 | 287 => return Err(ERROR_VALUE.into()),
            length_code => {
                if length_code > 285 {
                    return Err(ERROR_VALUE.into());
                }
                let (code_base_length, width_length) = LENGTH_TABLE[length_code as usize - 257];
                let bits_length = reader.read_bits(width_length)?;

                let decoded = self.distance.decode(reader)?;
                let (code_base_distance, width_distance) = DISTANCE_TABLE[decoded as usize];
                let bits_distance = reader.read_bits(width_distance)?;

                Code::Pointer {
                    length: code_base_length + bits_length as u8,
                    distance: code_base_distance + bits_distance,
                }
            }
        })
    }
}
// Functions.
pub fn calc_optimal_max_width(frequencies: &[usize]) -> u8 {
    let mut heap = BinaryHeap::new();
    for freq in frequencies.iter().filter(|f| **f > 0) {
        let weight = -(*freq as isize);
        heap.push((weight, 0 as u8));
    }
    while heap.len() > 1 {
        let (weight1, width1) = heap.pop().unwrap();
        let (weight2, width2) = heap.pop().unwrap();
        heap.push((weight1 + weight2, 1 + std::cmp::max(width1, width2)));
    }
    let max_bitwidth = heap.pop().map_or(0, |x| x.1);
    std::cmp::max(1, max_bitwidth)
}
/// Reference: [A Fast Algorithm for Optimal Length-Limited Huffman Codes][LenLimHuff.pdf]
///
/// [LenLimHuff.pdf]: https://www.ics.uci.edu/~dan/pubs/LenLimHuff.pdf
pub fn calc(max_width: u8, frequencies: &[usize]) -> Vec<u8> {
    // NOTE: unoptimized implementation
    let mut source = frequencies
        .iter()
        .enumerate()
        .filter(|(_, f)| **f > 0)
        .map(|(symbol, weight)| Node::single(symbol as u16, *weight))
        .collect::<Vec<_>>();
    source.sort_by_key(|o| o.weight);

    let weighted =
        (0..max_width - 1).fold(source.clone(), |w, _| merge(package(w), source.clone()));

    let mut code_widthes = vec![0; frequencies.len()];
    for symbol in package(weighted)
        .into_iter()
        .flat_map(|n| n.symbols.into_iter())
    {
        code_widthes[symbol as usize] += 1;
    }
    code_widthes
}
pub fn merge(x: Vec<Node>, y: Vec<Node>) -> Vec<Node> {
    let mut z = Vec::with_capacity(x.len() + y.len());
    let mut x = x.into_iter().peekable();
    let mut y = y.into_iter().peekable();
    loop {
        let x_weight = x.peek().map(|s| s.weight);
        let y_weight = y.peek().map(|s| s.weight);
        if x_weight.is_none() {
            z.extend(y);
            break;
        } else if y_weight.is_none() {
            z.extend(x);
            break;
        } else if x_weight < y_weight {
            z.push(x.next().unwrap());
        } else {
            z.push(y.next().unwrap());
        }
    }
    z
}
pub fn package(mut nodes: Vec<Node>) -> Vec<Node> {
    if nodes.len() >= 2 {
        let new_len = nodes.len() / 2;

        for i in 0..new_len {
            nodes[i] = std::mem::replace(&mut nodes[i * 2], Node::empty());
            let other = std::mem::replace(&mut nodes[i * 2 + 1], Node::empty());
            nodes[i].merge(other);
        }
        nodes.truncate(new_len);
    }
    nodes
}
// Main functions.
pub fn huffman_encode(v_in: &[Code], btype: BlockType) -> Vec<u8> {
    // Variable Initialization.
    let mut writer = Writer::new();

    // Algorithms.
    writer.write_bits(Bits { data: 1, width: 1 });
    writer.write_bits(Bits {
        data: btype as u16,
        width: 2,
    });
    let encoder = HuffmanEncoder::new(btype, v_in);
    if btype == BlockType::Dynamic {
        encoder.save(&mut writer);
    }
    for code in v_in {
        encoder.encode(&mut writer, *code);
    }
    writer.finish()
}

pub fn huffman_decode(v_in: &[u8]) -> Result<Vec<Code>, String> {
    // Variable Initialization.
    let mut reader = Reader::new(v_in);
    let mut bfinal = 0;
    let mut v_out = Vec::new();

    // Algorithms.
    while bfinal == 0 {
        bfinal = reader.read_bits(1)?;
        match reader.read_bits(2)? {
            0b11 => return Err(ERROR_RESERVED.into()),
            0b00 => {
                reader.reset();
                let len = reader.read_u16()?;
                let nlen = reader.read_u16()?;
                if !len != nlen {
                    return Err(ERROR_COMPLEMENT.into());
                }
                for _ in (0..).take(len as usize) {
                    v_out.push(Code::Literal(reader.read_u8()?));
                }
            }
            btype => {
                let decoder = HuffmanDecoder::new(BlockType::from(btype), &mut reader)?;
                loop {
                    let x = decoder.decode(&mut reader)?;
                    match x {
                        Code::EndOfBlock => break,
                        _ => v_out.push(x),
                    }
                }
            }
        }
    }
    Ok(v_out)
}
