use devker::huffman::BlockType;
use devker::deflate::{deflate, inflate};
use libflate::deflate::{Decoder, EncodeOptions, Encoder};
use std::io::prelude::*;
const STEP: usize = 3;

fn block0(v_in: &[u8]) {
    let now = std::time::Instant::now();
    let options = EncodeOptions::new().fixed_huffman_codes();
    let mut encoder = Encoder::with_options(Vec::new(), options);
    std::io::copy(&mut &v_in[..], &mut encoder).unwrap();
    let encoded = encoder.finish().into_result().unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!("time: {}.{:09} s - deflate - size: {}", sec, subsec, encoded.len());

    let now = std::time::Instant::now();
    let mut decoder = Decoder::new(&encoded[..]);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!("time: {}.{:09} s - inflate - size: {}", sec, subsec, decoded.len());

    assert_eq!(v_in, &decoded[..]);
}

fn block1(v_in: &[u8]) {
    let now = std::time::Instant::now();
    let encoded = deflate(&v_in, BlockType::Fixed);
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!("time: {}.{:09} s - deflate - size: {}", sec, subsec, encoded.len());

    let now = std::time::Instant::now();
    let decoded = inflate(&encoded).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!("time: {}.{:09} s - inflate - size: {}", sec, subsec, decoded.len());
    
    assert_eq!(v_in, &decoded[..]);
}

fn main() {
    let mut file = std::fs::File::open("./enwiki-latest-all-titles-in-ns0").unwrap();
    let mut v = String::new();
    file.read_to_string(&mut v).unwrap();
    let v_in = v.into_bytes();

    println!("libflate");
    for _ in 0..STEP {
        block0(&v_in);
    }
    println!("core");
    for _ in 0..STEP {
        block1(&v_in);
    }
}
