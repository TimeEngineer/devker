//! # Support
//!
//! * Deflate/Inflate: [`deflate`]
//! * Zlib: [`zlib`]
//!
//! [`deflate`]: deflate/index.html
//! [`zlib`]: zlib/index.html

pub mod prelude {
    pub use crate::btype::BlockType;
    pub use crate::cache::Cache;
    pub use crate::deflate::{deflate, inflate, inflate_to};
    pub use crate::zlib::{zlib_decode, zlib_decode_to, zlib_encode};
}

mod adler32;
mod bits;
pub mod btype;
pub mod cache;
mod code;
pub mod deflate;
mod huffman;
mod lzss;
pub mod zlib;
