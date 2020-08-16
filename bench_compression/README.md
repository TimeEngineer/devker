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
time: 6.481724966 s - size: 116943701 - deflate 
time: 1.461838843 s - size: 329839116 - inflate
time: 6.385758042 s - size: 116943701 - deflate 
time: 1.459047555 s - size: 329839116 - inflate
time: 6.387020922 s - size: 116943701 - deflate 
time: 1.471621869 s - size: 329839116 - inflate
devker
time: 2.079220781 s - size: 125813886 - deflate
time: 1.390934409 s - size: 329839116 - inflate
time: 2.087242892 s - size: 125813886 - deflate
time: 1.396040085 s - size: 329839116 - inflate
time: 2.086336990 s - size: 125813886 - deflate
time: 1.390870687 s - size: 329839116 - inflate
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
