//! # Zlib encode/decode
//!
//! ## Examples
//!
//! ### Easy to use.
//! ```
//! use devker::prelude::{zlib_decode, zlib_encode, BlockType, Cache};
//!
//! let mut cache = Cache::new();
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! // Encode.
//! let encoded = zlib_encode(&v_in, BlockType::Fixed, &mut cache);
//! // Decode.
//! let decoded = zlib_decode(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//! ```
//!
//! ### Reusable cache.
//! ```
//! use devker::prelude::{zlib_decode, zlib_encode, BlockType, Cache};
//!
//! let mut cache = Cache::new();
//!
//! // First try.
//!
//! let v = String::from("Hello world, this is a wonderful world !");
//! let v_in = v.into_bytes();
//!
//! let encoded = zlib_encode(&v_in, BlockType::Fixed, &mut cache);
//! let decoded = zlib_decode(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//!
//! // Another try.
//!
//! let v = String::from("The cache can be reused !");
//! let v_in = v.into_bytes();
//!
//! let encoded = zlib_encode(&v_in, BlockType::Fixed, &mut cache);
//! let decoded = zlib_decode(&encoded, &mut cache).unwrap();
//! assert_eq!(v_in, decoded);
//! ```

// Import.
use crate::adler32::Adler32;
use crate::prelude::{deflate, inflate, inflate_to, BlockType, Cache};
use std::convert::TryInto;
// Constants.
const ERROR_ADLER32: &str = "Zlib checksum error";
const ERROR_DEFLATE: &str = "Zlib only supports deflate compression algorithm";
const ERROR_DICT: &str = "Zlib dictionaries is not implemented";
const ERROR_FCHECK: &str = "Fcheck must be multiple of 31";
const ERROR_LENGTH: &str = "Zlib's header is missing";
const HEADER_LEN: usize = 2;
const ADLER_LEN: usize = 4;
const METHOD_DEFLATE: u8 = 8;
// Main functions.
pub fn zlib_encode(v_in: &[u8], btype: BlockType, cache: &mut Cache) -> Vec<u8> {
    // Variable initialization.
    let cmf = 0x78;
    let flevel = 2;
    let fcheck = 28;
    let flg = flevel << 6 | fcheck;
    let mut adler32 = Adler32::new();

    // Algorithm.
    let mut data = deflate(v_in, btype, cache);
    adler32.update(&v_in);
    let mut v_out = Vec::with_capacity(data.len() + 2 + 4);
    v_out.extend(&[cmf, flg]);
    v_out.append(&mut data);
    v_out.extend(&adler32.checksum());
    v_out
}

pub fn zlib_decode(v_in: &[u8], cache: &mut Cache) -> Result<Vec<u8>, String> {
    // Conditions.
    if v_in.len() < HEADER_LEN + ADLER_LEN {
        return Err(ERROR_LENGTH.into());
    }
    let cmf = v_in[0];
    let flg = v_in[1];
    if (cmf & 0x0F) != METHOD_DEFLATE {
        return Err(ERROR_DEFLATE.into());
    }
    if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return Err(ERROR_FCHECK.into());
    }
    if (flg & 0b100_000) > 0 {
        return Err(ERROR_DICT.into());
    }
    // Variable initialization.
    let mut adler32 = Adler32::new();

    // Algorithm.
    let v_out = inflate(&v_in[HEADER_LEN..v_in.len() - ADLER_LEN], cache)?;
    adler32.update(&v_out);
    let _adler32: [u8; 4] = v_in[v_in.len() - ADLER_LEN..].try_into().unwrap();
    if adler32.checksum() != _adler32 {
        return Err(ERROR_ADLER32.into());
    }
    Ok(v_out)
}

pub fn zlib_decode_to(v_in: &[u8], cache: &mut Cache, v_out: &mut [u8]) -> Result<(), String> {
    // Conditions.
    if v_in.len() < HEADER_LEN + ADLER_LEN {
        return Err(ERROR_LENGTH.into());
    }
    let cmf = v_in[0];
    let flg = v_in[1];
    if (cmf & 0x0F) != METHOD_DEFLATE {
        return Err(ERROR_DEFLATE.into());
    }
    if ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return Err(ERROR_FCHECK.into());
    }
    if (flg & 0b100_000) > 0 {
        return Err(ERROR_DICT.into());
    }
    // Variable initialization.
    let mut adler32 = Adler32::new();

    // Algorithm.
    inflate_to(&v_in[HEADER_LEN..v_in.len() - ADLER_LEN], cache, v_out)?;
    adler32.update(v_out);
    let _adler32: [u8; 4] = v_in[v_in.len() - ADLER_LEN..].try_into().unwrap();
    if adler32.checksum() != _adler32 {
        return Err(ERROR_ADLER32.into());
    }
    Ok(())
}
