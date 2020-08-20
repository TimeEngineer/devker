//! # Support
//!
//! * Deflate/Inflate: [`deflate`]
//!
//! [`deflate`]: deflate/index.html

pub mod prelude {
    pub use crate::btype::BlockType;
    pub use crate::cache::Cache;
    pub use crate::deflate::{deflate, inflate, inflate_to};
}

mod bits;
pub mod btype;
pub mod cache;
mod code;
pub mod deflate;
mod huffman;
mod lzss;
