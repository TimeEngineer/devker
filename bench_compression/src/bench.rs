use devker::prelude::{deflate, inflate, BlockType, Cache};
use libflate::deflate::{Decoder, EncodeOptions, Encoder};
use std::io::prelude::*;
use rand::{thread_rng, Rng};
const MIN_SIZE: usize = 10;
const MAX_SIZE: usize = 30;
const NTIME: usize = 2;

fn block0(v_in: &[u8]) -> (f64, f64) {
    let now = std::time::Instant::now();
    let options = EncodeOptions::new().fixed_huffman_codes();
    let mut encoder = Encoder::with_options(Vec::new(), options);
    std::io::copy(&mut &v_in[..], &mut encoder).unwrap();
    let encoded = encoder.finish().into_result().unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();

    let time0 = sec as f64 * 1_000_000_000. + subsec as f64;

    let now = std::time::Instant::now();
    let mut decoder = Decoder::new(&encoded[..]);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();

    let time1 = sec as f64 * 1_000_000_000. + subsec as f64;

    assert_eq!(v_in, &decoded[..]);
    (time0, time1)
}

fn block1(v_in: &[u8], cache: &mut Cache) -> (f64, f64) {
    let now = std::time::Instant::now();
    let encoded = deflate(&v_in, BlockType::Fixed, cache);
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();
    
    let time0 = sec as f64 * 1_000_000_000. + subsec as f64;

    let now = std::time::Instant::now();
    let decoded = inflate(&encoded, cache).unwrap();
    let sec = now.elapsed().as_secs();
    let subsec = now.elapsed().subsec_nanos();

    let time1 = sec as f64 * 1_000_000_000. + subsec as f64;

    assert_eq!(v_in, &decoded[..]);
    (time0, time1)
}

fn main() {
    let mut cache = Cache::new();
    let mut file0 = std::fs::File::create("bench/libflate_deflate.csv").unwrap();
    let mut file1 = std::fs::File::create("bench/libflate_inflate.csv").unwrap();
    let mut file2 = std::fs::File::create("bench/devker_deflate.csv").unwrap();
    let mut file3 = std::fs::File::create("bench/devker_inflate.csv").unwrap();

    for i in MIN_SIZE..MAX_SIZE {
        let size = 1 << i;
        let mut v_in = vec![0u8; size];
        thread_rng().fill(&mut v_in[..]);
        let mut time0 = 0.;
        let mut time1 = 0.;
        let mut time2 = 0.;
        let mut time3 = 0.;

        for _ in 0..NTIME {
            let time = block0(&v_in);
            time0 += time.0;
            time1 += time.1;
            let time = block1(&v_in, &mut cache);
            time2 += time.0;
            time3 += time.1;
        }

        time0 /= NTIME as f64;
        time1 /= NTIME as f64;
        time2 /= NTIME as f64;
        time3 /= NTIME as f64;

        let _ = write!(file0, "{},{}\n", size, time0);
        let _ = write!(file1, "{},{}\n", size, time1);
        let _ = write!(file2, "{},{}\n", size, time2);
        let _ = write!(file3, "{},{}\n", size, time3);
    }
}