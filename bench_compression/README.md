# Report

CPU info:
---------
AMD Ryzen 7 3800X 8-Core Processor

RAM info:
---------
CORSAIR VENGEANCE LPX 16 GB (2 x 8) 3.6 GHz C18

wiki
----

```bash
$ cd core/bench_compression/
$ curl -O https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-all-titles-in-ns0.gz
$ gzip -d enwiki-latest-all-titles-in-ns0.gz

$ cargo run --release --bin wiki

libflate
time: 6.711064413 s - deflate - size: 116943701
time: 1.537833225 s - inflate - size: 329839116
time: 6.588826391 s - deflate - size: 116943701
time: 1.536978554 s - inflate - size: 329839116
time: 6.595836837 s - deflate - size: 116943701
time: 1.538615950 s - inflate - size: 329839116
core
time: 2.540604854 s - deflate - size: 116730274
time: 2.459512773 s - inflate - size: 329839116
time: 2.543647852 s - deflate - size: 116730274
time: 2.476856540 s - inflate - size: 329839116
time: 2.535002666 s - deflate - size: 116730274
time: 2.483141591 s - inflate - size: 329839116
```
