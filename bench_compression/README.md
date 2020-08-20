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
$ cd devker/bench_compression/
$ curl -O https://dumps.wikimedia.org/enwiki/latest/enwiki-latest-all-titles-in-ns0.gz
$ gzip -d enwiki-latest-all-titles-in-ns0.gz

$ cargo run --release --bin wiki

libflate
time: 6.375271453 s - size: 116943701 - deflate 
time: 1.507718897 s - size: 329839116 - inflate
time: 6.367969073 s - size: 116943701 - deflate 
time: 1.509135068 s - size: 329839116 - inflate
time: 6.371911262 s - size: 116943701 - deflate 
time: 1.508372329 s - size: 329839116 - inflate
devker
time: 2.109853021 s - size: 125813886 - deflate
time: 1.504122062 s - size: 329839116 - inflate
time: 1.502997270 s - size: 329839116 - inflate_to
time: 2.108059192 s - size: 125813886 - deflate
time: 1.508975561 s - size: 329839116 - inflate
time: 1.505576031 s - size: 329839116 - inflate_to
time: 2.108160049 s - size: 125813886 - deflate
time: 1.507358968 s - size: 329839116 - inflate
time: 1.503677668 s - size: 329839116 - inflate_to
```

bench
-----

```bash
$ cd devker/bench_compression/

$ cargo run --release --bin bench

$ python3 plot.py
```

![alt text](https://github.com/TimeEngineer/devker/blob/master/bench_compression/bench/bench.png "bench")
![alt text](https://github.com/TimeEngineer/devker/blob/master/bench_compression/bench/deflate.png "deflate")
![alt text](https://github.com/TimeEngineer/devker/blob/master/bench_compression/bench/inflate.png "inflate")
