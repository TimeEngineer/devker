//! # LZSS

// Import.
use crate::code::Code;
// Constants.
const MAX_WINDOW_LENGTH: usize = 0x8000;
const MAX_LENGTH: usize = 0x100;
const ERROR_ENDOFBLOCK: &str = "End Of Block is not supposed to be there.";
const ERROR_POSITION: &str = "One distance is greater than current index.";
// Structures.
struct PrefixTable<'a>(&'a mut [i32; 0x10000]);
// Implementations.
impl<'a> PrefixTable<'a> {
    fn new(array: &'a mut [i32; 0x10000]) -> Self {
        for x in array.iter_mut() {
            *x = -1;
        }
        Self(array)
    }
    fn insert(&mut self, buf: &[u8], value: i32) -> Option<i32> {
        let key = prefix(buf);
        let old = self.0[key];
        self.0[key] = value;
        if !old.is_negative() {
            return Some(old);
        }
        None
    }
    fn overwrite(&mut self, buf: &[u8], value: i32) {
        let key = prefix(buf);
        self.0[key] = value;
    }
}
// Functions.
fn prefix(buf: &[u8]) -> usize {
    let mut array = [0; 8];
    (&mut array[6..8]).copy_from_slice(&buf[0..2]);
    usize::from_be_bytes(array)
}
fn longest_match(buf: &[u8], d: usize) -> usize {
    buf[d..]
        .iter()
        .zip(buf)
        .take(MAX_LENGTH)
        .take_while(|(x, y)| *x == *y)
        .count()
}
pub fn extend(buf: &mut Vec<u8>, mut d: usize, mut l: usize) -> Result<(), String> {
    if buf.len() < d {
        return Err(ERROR_POSITION.into());
    }
    let start = buf.len() - d;
    buf.reserve(l);

    // Copy bytes fastly
    while l >= d {
        unsafe {
            let len = buf.len();
            std::ptr::copy_nonoverlapping(buf.get_unchecked(start), buf.get_unchecked_mut(len), d);
            buf.set_len(len + d);
        }
        l -= d;
        d *= 2;
    }

    // Copy the last remaining bytes
    unsafe {
        let len = buf.len();
        std::ptr::copy_nonoverlapping(buf.get_unchecked(start), buf.get_unchecked_mut(len), l);
        buf.set_len(len + l);
    }

    Ok(())
}
// Main functions.
pub fn lzss_encode(v_in: &[u8], buf: &mut [i32; 0x10000]) -> Vec<Code> {
    // Variable initialization.
    let end = std::cmp::max(3, v_in.len()) - 3;
    let mut prefix_table = PrefixTable::new(buf);
    let mut v_out = Vec::new();
    let mut i = 0;

    // Algorithm.
    while i < end {
        let matched = prefix_table.insert(&v_in[i..], i as i32);
        if let Some(j) = matched.map(|j| j as usize) {
            let distance = i - j;
            if distance <= MAX_WINDOW_LENGTH {
                let len = longest_match(&v_in[j + 2..], distance);
                let length = std::cmp::min(len + 2, end + 1 - i);
                if len > 2 {
                    for k in (i..).take(length).skip(1) {
                        prefix_table.overwrite(&v_in[k..], k as i32);
                    }
                    i += length;
                    let distance = distance as u16;
                    let length = (length - 3) as u8;
                    v_out.push(Code::Pointer { distance, length });
                    continue;
                }
            }
        }
        v_out.push(Code::Literal(v_in[i]));
        i += 1;
    }
    for x in &v_in[i..] {
        v_out.push(Code::Literal(*x));
    }
    v_out
}

#[allow(dead_code)]
pub fn lzss_decode(v_in: &[Code]) -> Result<Vec<u8>, String> {
    // Variable initialization.
    let mut v_out = Vec::new();

    // Algorithm.
    for &code in v_in {
        match code {
            Code::EndOfBlock => return Err(ERROR_ENDOFBLOCK.into()),
            Code::Literal(a) => v_out.push(a),
            Code::Pointer {
                distance: d,
                length: l,
            } => extend(&mut v_out, d as usize, l as usize + 3)?,
        }
    }
    Ok(v_out)
}
