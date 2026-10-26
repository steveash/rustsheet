"""Bin rows of need_*.tsv by a key function of (E, t) and count consistent bins (non-empty intersection of the
needed-offset intervals), against a shuffled null.  Usage: kbin.py need_x.tsv [--a1] [--bfix]"""
import collections
import math
import random
import sys

import mpmath
from mpmath import mpf

rows = []
for fn in [f for f in sys.argv[1:] if not f.startswith("--")]:
    for line in open(fn):
        x, a, b, E, t, lo, hi = line.rstrip("\n").split("\t")
        if lo == "None" or ("--a1" in sys.argv and float(a) != 1.0):
            continue
        rows.append(dict(x=float(x), b=float(b), E=mpf(E), t=mpf(t), lo=int(lo), hi=int(hi)))

KEYS = {
    "floorE": lambda r: math.floor(r["E"]),
    "nintE": lambda r: int(mpmath.nint(r["E"])),
    "nint_t": lambda r: int(mpmath.nint(r["t"])),
    "floor_t": lambda r: math.floor(r["t"]),
    "E*8": lambda r: math.floor(r["E"] * 8),
    "E*64": lambda r: math.floor(r["E"] * 64),
    "t*16": lambda r: math.floor(r["t"] * 16),
    "t*256": lambda r: math.floor(r["t"] * 256),
    "b": lambda r: r["b"],
}


def consist(rs, key):
    bins = collections.defaultdict(lambda: [-10**9, 10**9, 0])
    for r in rs:
        bn = bins[key(r)]
        bn[0], bn[1], bn[2] = max(bn[0], r["lo"]), min(bn[1], r["hi"]), bn[2] + 1
    multi = [v for v in bins.values() if v[2] >= 2]
    return len(multi), sum(v[0] <= v[1] for v in multi), sum(v[2] for v in multi)


random.seed(2)
for nm, key in KEYS.items():
    real = consist(rows, key)
    sh = [dict(r) for r in rows]
    iv = [(r["lo"], r["hi"]) for r in sh]
    random.shuffle(iv)
    for r, (lo, hi) in zip(sh, iv):
        r["lo"], r["hi"] = lo, hi
    null = consist(sh, key)
    print(f"{nm:8s} bins {real[0]:5d} rows {real[2]:6d} consistent {real[1]:5d}   null {null[1]:5d}")
