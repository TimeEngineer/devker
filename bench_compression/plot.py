import numpy as numpy
import matplotlib.pyplot as plt

BENCH_PATH = "bench/"

def bench(path, label):
    x = []
    y = []
    fo = open(path, "r")
    for line in fo:
        (xi, yi) = line.split(',')
        x.append(int(xi))
        y.append(float(yi)/1000.)

    line, = ax.loglog(x, y, basex=2)
    line.set_label(label)

def bench_linear(path, label):
    x = []
    y = []
    fo = open(path, "r")
    for line in fo:
        (xi, yi) = line.split(',')
        x.append(int(xi))
        y.append(float(yi)/float(xi)/1000)

    line, = ax.loglog(x, y, basex=2)
    line.set_label(label)

def bench_square(path, label):
    x = []
    y = []
    fo = open(path, "r")
    for line in fo:
        (xi, yi) = line.split(',')
        x.append(int(xi))
        y.append(float(yi)/(float(xi)*float(xi)))

    line, = ax.loglog(x, y, basex=2)
    line.set_label(label/1000.)

fig = plt.figure()

ax = plt.subplot(111)

# bench(BENCH_PATH + "libflate_deflate.csv", "libflate - deflate")
# bench(BENCH_PATH + "libflate_inflate.csv", "libflate - inflate")
# bench(BENCH_PATH + "devker_deflate.csv", "devker - deflate")
# bench(BENCH_PATH + "devker_inflate.csv", "devker - inflate")
# bench(BENCH_PATH + "devker_inflate_to.csv", "devker - inflate_to")
bench_linear(BENCH_PATH + "libflate_deflate.csv", "libflate - deflate / size")
bench_linear(BENCH_PATH + "libflate_inflate.csv", "libflate - inflate / size")
bench_linear(BENCH_PATH + "devker_deflate.csv", "devker - deflate / size")
bench_linear(BENCH_PATH + "devker_inflate.csv", "devker - inflate / size")
bench_linear(BENCH_PATH + "devker_inflate_to.csv", "devker - inflate_to / size")

ax.set_title('Benchmark [ms]', fontsize=16)
ax.set_xlabel('size', fontsize=16)
ax.set_ylabel('time', fontsize=16)
ax.yaxis.set_tick_params(labelsize=12)
ax.legend(fontsize=12, loc='upper left')

plt.show()