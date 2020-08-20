# DevKER

[![devker](https://img.shields.io/crates/v/devker.svg)](https://crates.io/crates/devker)
[![Documentation](https://docs.rs/devker/badge.svg)](https://docs.rs/devker)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Exemple
-------

- Easy to use.
```Rust
use devker::prelude::{deflate, inflate, BlockType, Cache};

let mut cache = Cache::new();
let v = String::from("Hello world, this is a wonderful world !");
let v_in = v.into_bytes();

// Encode.
let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
// Decode.
let decoded = inflate(&encoded, &mut cache).unwrap();
assert_eq!(v_in, decoded);
```

- Reusable cache.
```Rust
use devker::prelude::{deflate, inflate, BlockType, Cache};

let mut cache = Cache::new();

// First try.

let v = String::from("Hello world, this is a wonderful world !");
let v_in = v.into_bytes();

let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
let decoded = inflate(&encoded, &mut cache).unwrap();
assert_eq!(v_in, decoded);

// Another try.

let v = String::from("The cache can be reused !");
let v_in = v.into_bytes();

let encoded = deflate(&v_in, BlockType::Fixed, &mut cache);
let decoded = inflate(&encoded, &mut cache).unwrap();
assert_eq!(v_in, decoded);
```

Support
-------

- Deflate/Inflate (Only fixed is supported for deflate)

Note
----

For the moment, this crate is inspired by libflate.

Documentation
-------------

See [RustDoc Documentation](https://docs.rs/devker).

Installation
------------

Add following lines to your `Cargo.toml`:

```toml
[dependencies]
devker = "0"
```

Goal
----

In the future, this crate gathers most of the algorithms that I use for my projects. 

The goal is to have performance and no dependency, in order to fully control the source code.

References
----------

- DEFLATE: [RFC-1951](https://tools.ietf.org/html/rfc1951)
