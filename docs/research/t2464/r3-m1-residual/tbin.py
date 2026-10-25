"""Table test: bin rows by floor(frac(t) * N), t = E / ln 2, and intersect each bin's needed-offset intervals
(need_*.tsv). A table-driven exp (2^(j/N) entries with their own rounding errors) makes every bin consistent at
N >= its table size.  Usage: tbin.py need_a.tsv [need_b.tsv ...] [--a1]"""
import collections
import math
import sys

from mpmath import mpf

files = [f for f in sys.argv[1:] if not f.startswith("--")]
rows = []
for fn in files:
    for line in open(fn):
        x, a, b, E, t, lo, hi = line.rstrip("\n").split("\t")
        if lo == "None" or ("--a1" in sys.argv and float(a) != 1.0):
            continue
        rows.append((mpf(t), int(lo), int(hi)))
print("rows", len(rows))
for N in (32, 64, 128, 256, 512, 1024, 2048, 4096, 8192):
    bins = collections.defaultdict(lambda: [-10**9, 10**9, 0])
    for t, lo, hi in rows:
        fr = t - math.floor(t)
        k = int(fr * N)
        bn = bins[k]
        bn[0], bn[1], bn[2] = max(bn[0], lo), min(bn[1], hi), bn[2] + 1
    multi = [v for v in bins.values() if v[2] >= 2]
    ok = sum(v[0] <= v[1] for v in multi)
    rows_ok = sum(v[2] for v in multi if v[0] <= v[1])
    print(f"N={N:5d} bins with >=2 rows {len(multi):5d}, consistent {ok:5d} ({rows_ok} rows)")

# null: the same intervals with t shuffled
import random

random.seed(1)
ts = [r[0] for r in rows]
random.shuffle(ts)
rows = [(t, lo, hi) for t, (_, lo, hi) in zip(ts, rows)]
print("null (t shuffled)")
for N in (1024, 2048, 4096, 8192):
    bins = collections.defaultdict(lambda: [-10**9, 10**9, 0])
    for t, lo, hi in rows:
        k = int((t - math.floor(t)) * N)
        bn = bins[k]
        bn[0], bn[1], bn[2] = max(bn[0], lo), min(bn[1], hi), bn[2] + 1
    multi = [v for v in bins.values() if v[2] >= 2]
    print(f"N={N:5d} bins {len(multi):5d}, consistent {sum(v[0] <= v[1] for v in multi):5d}")
