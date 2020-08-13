//! # LZSS
//!
//! ## Examples
//!
//! ```
//! use devker::lzss::{lzss_decode, lzss_encode};
//!
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! // Encode
//! let encoded = lzss_encode(&v_in);
//! // Decode
//! let decoded = lzss_decode(&encoded).unwrap();
//! assert_eq!(v_in, decoded);
//! ```

// Import.
use crate::code::Code;
use std::collections::HashMap;
// Constants.
pub const MAX_WINDOW_LENGTH: usize = 0x8000;
pub const MAX_LENGTH: usize = 0xFF;
pub const ERROR_ENDOFBLOCK: &str = "End Of Block is not supposed to be there.";
pub const ERROR_POSITION: &str = "One distance is greater than current index.";
// Structures.
pub struct PrefixTableVec(Vec<isize>);
pub type PrefixTableHash = HashMap<usize, usize>;
pub enum PrefixTable {
    Small(PrefixTableHash),
    Large(PrefixTableVec),
}
// Implementations.
impl PrefixTableVec {
    pub fn new() -> Self {
        Self(vec![-1; 0x1000000])
    }
    pub fn insert(&mut self, key: usize, value: usize) -> Option<usize> {
        let old = self.0[key];
        self.0[key] = value as isize;
        if !old.is_negative() {
            return Some(old as usize);
        }
        None
    }
}
impl PrefixTable {
    pub fn new(len: usize) -> Self {
        if len < 0x80000 {
            Self::Small(HashMap::new())
        } else {
            Self::Large(PrefixTableVec::new())
        }
    }
    pub fn insert(&mut self, key: usize, value: usize) -> Option<usize> {
        match self {
            Self::Small(hash) => hash.insert(key, value),
            Self::Large(vec) => vec.insert(key, value),
        }
    }
}
// Functions.
pub fn prefix(buf: &[u8]) -> usize {
    let mut array = [0; 8];
    (&mut array[5..8]).copy_from_slice(&buf[0..3]);
    usize::from_be_bytes(array)
}
pub fn longest_match(buf: &[u8], i: usize, j: usize) -> usize {
    buf[i..]
        .iter()
        .zip(&buf[j..])
        .take(MAX_LENGTH)
        .take_while(|(x, y)| *x == *y)
        .count()
}
// Main functions.
pub fn lzss_encode(v_in: &[u8]) -> Vec<Code> {
    // Variable initialization.
    let v_len = v_in.len();
    let end = std::cmp::max(3, v_len) - 3;
    let mut v_out = Vec::new();
    let mut prefix_table = PrefixTable::new(v_len);
    let mut i = 0;

    // Algorithm.
    while i < end {
        let key = prefix(&v_in[i..]);
        let matched = prefix_table.insert(key, i);
        if let Some(j) = matched {
            let distance = i - j;
            if distance <= MAX_WINDOW_LENGTH {
                let length = 3 + longest_match(&v_in, i + 3, j + 3);
                let length = std::cmp::min(length, end - i + 1);
                for k in (i..).take(length).skip(1) {
                    prefix_table.insert(prefix(&v_in[k..]), k);
                }
                i += length;
                let distance = distance as u16;
                let length = (length - 3) as u8;
                v_out.push(Code::Pointer { distance, length });
                continue;
            }
        }
        v_out.push(Code::Literal(v_in[i]));
        i += 1;
    }
    v_out.reserve(v_in[i..].len());
    for x in &v_in[i..] {
        v_out.push(Code::Literal(*x));
    }
    v_out
}

pub fn lzss_decode(v_in: &[Code]) -> Result<Vec<u8>, String> {
    // Variable initialization.
    let mut v_out = Vec::new();
    let mut i = 0;

    // Algorithm.
    for code in v_in {
        match *code {
            Code::EndOfBlock => return Err(ERROR_ENDOFBLOCK.into()),
            Code::Literal(a) => {
                v_out.push(a);
                i += 1;
            }
            Code::Pointer {
                distance: d,
                length: l,
            } => {
                let d = d as usize;
                let l = l as usize + 3;
                if i < d {
                    return Err(ERROR_POSITION.into());
                }
                let p = i - d;
                for j in (p..).take(l) {
                    v_out.push(v_out[j]);
                }
                i += l;
            }
        }
    }
    Ok(v_out)
}
