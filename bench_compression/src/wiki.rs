use devker::prelude::{deflate, inflate, inflate_to, BlockType, Cache};
use libflate::deflate::{Decoder, EncodeOptions, Encoder};
use std::io::prelude::*;
const NTIME: usize = 3;

fn block0(v_in: &[u8]) {
    let now = std::time::Instant::now();
    let options = EncodeOptions::new().fixed_huffman_codes();
    let mut encoder = Encoder::with_options(Vec::new(), options);
    std::io::copy(&mut &v_in[..], &mut encoder).unwrap();
    let encoded = encoder.finish().into_result().unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!(
        "time: {}.{:09} s - size: {} - deflate ",
        sec,
        subsec,
        encoded.len()
    );

    let mut decoded = Vec::with_capacity(v_in.len());
    let now = std::time::Instant::now();
    let mut decoder = Decoder::new(&encoded[..]);
    decoder.read_to_end(&mut decoded).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!(
        "time: {}.{:09} s - size: {} - inflate",
        sec,
        subsec,
        decoded.len()
    );
    assert_eq!(v_in, &decoded[..]);
}

fn block1(v_in: &[u8], cache: &mut Cache) {
    let now = std::time::Instant::now();
    let encoded = deflate(&v_in, BlockType::Fixed, cache);
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!(
        "time: {}.{:09} s - size: {} - deflate",
        sec,
        subsec,
        encoded.len()
    );

    let now = std::time::Instant::now();
    let decoded = inflate(&encoded, cache).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!(
        "time: {}.{:09} s - size: {} - inflate",
        sec,
        subsec,
        decoded.len()
    );
    assert_eq!(v_in, &decoded[..]);

    let mut decoded = vec![0; v_in.len()];
    let now = std::time::Instant::now();
    inflate_to(&encoded, cache, &mut decoded).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    println!(
        "time: {}.{:09} s - size: {} - inflate_to",
        sec,
        subsec,
        decoded.len()
    );
    assert_eq!(v_in, &decoded[..]);
}

fn main() {
    let mut file = std::fs::File::open("./enwiki-latest-all-titles-in-ns0").unwrap();
    let mut v = String::new();
    file.read_to_string(&mut v).unwrap();
    let v_in = v.into_bytes();
    let mut cache = Cache::new();

    println!("libflate");
    for _ in 0..NTIME {
        block0(&v_in);
    }
    println!("devker");
    for _ in 0..NTIME {
        block1(&v_in, &mut cache);
    }
}
