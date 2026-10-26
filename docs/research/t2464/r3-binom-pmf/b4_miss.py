"""Inspect one block's misses under a chain: each cell's admissible C interval vs the block's best C.
Usage: b4_miss.py <probe> <fn> <block> <chain> [knobs]"""
import sys

import numpy as np

import blib as B

name, fn, blk, ch = sys.argv[1:5]
o = eval(sys.argv[5]) if len(sys.argv) > 5 else dict(mr="xx")
rows = B.load_probe(name)
idx = [i for i, r in enumerate(rows) if r["fn"] == fn and r["blk"] == blk]
f = B.fit_block_g(rows, idx, o, ch) if ":" in ch else B.fit_block(rows, idx, o, ch)
c = f["off_all"]
print("all", f["all"], "/", f["n"], "C off", c)
for i in idx:
    iv = f["ivs"][i]
    r = rows[i]
    b1, b2, br = f["X"][i]
    q = B.st(1 - np.longdouble(r["p"]))
    ok = any(lo <= c <= hi for lo, hi in iv)
    print(f"{r['p']!r:24} {br} {'ok ' if ok else 'MISS'} iv={[(round(lo, 2), round(hi, 2)) for lo, hi in iv]} "
          f"b1={float(b1):.6f} b2={float(b2):.6f} np={float(np.longdouble(r['n'])*np.longdouble(r['p'])):.17g}")
