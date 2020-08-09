# DevKer

[![devker](https://img.shields.io/crates/v/devker.svg)](https://crates.io/crates/devker)
[![Documentation](https://docs.rs/devker/badge.svg)](https://docs.rs/devker)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Support
-------

- LZSS
- Huffman's coding
- Deflate/Inflate

Note
----

For the moment, this crate is inspired by libflate, notably for the dynamic Huffman's coding.
It has a slightly different architecture but the goal is mainly to learn and implement a PNG crate which requires the deflate algorithm.

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