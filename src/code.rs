// Constants.
pub const END_OF_BLOCK: u16 = 256;
pub const FIXED_LITERAL_BIT_CODE_TABLE: [(std::ops::Range<u16>, u8, u16); 4] = [
    (000..144, 8, 0b0_0011_0000),
    (144..256, 9, 0b1_1001_0000),
    (256..280, 7, 0b0_0000_0000),
    (280..288, 8, 0b0_1100_0000),
];
pub const LENGTH_TABLE: [(u8, u8); 29] = [
    (0, 0),
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 0),
    (6, 0),
    (7, 0),
    (8, 1),
    (10, 1),
    (12, 1),
    (14, 1),
    (16, 2),
    (20, 2),
    (24, 2),
    (28, 2),
    (32, 3),
    (40, 3),
    (48, 3),
    (56, 3),
    (64, 4),
    (80, 4),
    (96, 4),
    (112, 4),
    (128, 5),
    (160, 5),
    (192, 5),
    (224, 5),
    (255, 0),
];
pub const DISTANCE_TABLE: [(u16, u8); 30] = [
    (1, 0),
    (2, 0),
    (3, 0),
    (4, 0),
    (5, 1),
    (7, 1),
    (9, 2),
    (13, 2),
    (17, 3),
    (25, 3),
    (33, 4),
    (49, 4),
    (65, 5),
    (97, 5),
    (129, 6),
    (193, 6),
    (257, 7),
    (385, 7),
    (513, 8),
    (769, 8),
    (1025, 9),
    (1537, 9),
    (2049, 10),
    (3073, 10),
    (4097, 11),
    (6145, 11),
    (8193, 12),
    (12_289, 12),
    (16_385, 13),
    (24_577, 13),
];
// Structures.
#[derive(Debug, Clone, Copy)]
pub enum Code {
    EndOfBlock,
    Literal(u8),
    Pointer { distance: u16, length: u8 },
}
// Implementations.
impl Code {
    pub fn literal_code(&self) -> u16 {
        match *self {
            Code::Literal(x) => x as u16,
            Code::EndOfBlock => END_OF_BLOCK,
            Code::Pointer { length: l, .. } => match l {
                0x00..=0x07 => 257 + l as u16,
                0x08..=0x0F => 265 + (l as u16 - 0x08) / 0x02,
                0x10..=0x1F => 269 + (l as u16 - 0x10) / 0x04,
                0x20..=0x3F => 273 + (l as u16 - 0x20) / 0x08,
                0x40..=0x7F => 277 + (l as u16 - 0x40) / 0x10,
                0x80..=0xFE => 281 + (l as u16 - 0x80) / 0x20,
                0xFF => 285,
            },
        }
    }
    pub fn extra_length(&self) -> Option<(u8, u16)> {
        if let Code::Pointer { length: l, .. } = *self {
            match l {
                0x00..=0x07 | 0xFF => None,
                0x08..=0x0F => Some((1, (l as u16 - 0x08) % 0x02)),
                0x10..=0x1F => Some((2, (l as u16 - 0x10) % 0x04)),
                0x20..=0x3F => Some((3, (l as u16 - 0x20) % 0x08)),
                0x40..=0x7F => Some((4, (l as u16 - 0x40) % 0x10)),
                0x80..=0xFE => Some((5, (l as u16 - 0x80) % 0x20)),
            }
        } else {
            None
        }
    }
    pub fn distance_code(&self) -> Option<(u16, u8, u16)> {
        if let Code::Pointer { distance: d, .. } = *self {
            let d = d - 1;
            match d {
                0x0000..=0x0003 => Some((d, 0, 0)),
                0x0004..=0x0007 => {
                    let (div, rem) = div_rem(d - 0x0004, 0x0002);
                    Some((4 + div, 1, rem))
                }
                0x0008..=0x000F => {
                    let (div, rem) = div_rem(d - 0x0008, 0x0004);
                    Some((6 + div, 2, rem))
                }
                0x0010..=0x001F => {
                    let (div, rem) = div_rem(d - 0x0010, 0x0008);
                    Some((8 + div, 3, rem))
                }
                0x0020..=0x003F => {
                    let (div, rem) = div_rem(d - 0x0020, 0x0010);
                    Some((10 + div, 4, rem))
                }
                0x0040..=0x007F => {
                    let (div, rem) = div_rem(d - 0x0040, 0x0020);
                    Some((12 + div, 5, rem))
                }
                0x0080..=0x00FF => {
                    let (div, rem) = div_rem(d - 0x0080, 0x0040);
                    Some((14 + div, 6, rem))
                }
                0x0100..=0x01FF => {
                    let (div, rem) = div_rem(d - 0x0100, 0x0080);
                    Some((16 + div, 7, rem))
                }
                0x0200..=0x03FF => {
                    let (div, rem) = div_rem(d - 0x0200, 0x0100);
                    Some((18 + div, 8, rem))
                }
                0x0400..=0x07FF => {
                    let (div, rem) = div_rem(d - 0x0400, 0x0200);
                    Some((20 + div, 9, rem))
                }
                0x0800..=0x0FFF => {
                    let (div, rem) = div_rem(d - 0x0800, 0x0400);
                    Some((22 + div, 10, rem))
                }
                0x1000..=0x1FFF => {
                    let (div, rem) = div_rem(d - 0x1000, 0x0800);
                    Some((24 + div, 11, rem))
                }
                0x2000..=0x3FFF => {
                    let (div, rem) = div_rem(d - 0x2000, 0x1000);
                    Some((26 + div, 12, rem))
                }
                0x4000..=0x7FFF => {
                    let (div, rem) = div_rem(d - 0x4000, 0x2000);
                    Some((28 + div, 13, rem))
                }
                _ => unimplemented!(),
            }
        } else {
            None
        }
    }
}
// Functions.
fn div_rem(a: u16, b: u16) -> (u16, u16) {
    (a / b, a % b)
}
