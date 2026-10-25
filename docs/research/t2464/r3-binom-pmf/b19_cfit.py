"""Fitted stirlerr constant per block under out = fl(EXP_ext(lc) * F), lc = lc_chain(C, b1, b2), F = fx_F(F flags):
the best-overlap count and C's offset (2^-53 units) from the exact Cs and from R's table/series Cs.
Usage: b19_cfit.py <probe> <fn> <block substring> [F flags] [lc spec] [knobs]"""
import sys

import mpmath

import blib as B
from fractions import Fraction
B.Fraction = Fraction

LDB = B.LDB
name, fn, sub = sys.argv[1:4]
Ff = sys.argv[4] if len(sys.argv) > 4 else "xxxe"
spec = sys.argv[5] if len(sys.argv) > 5 else "R:ex"
o = dict(mr="xx", fin="mul", F=Ff)
if len(sys.argv) > 6:
    o.update(eval(sys.argv[6]))
rows = B.load_probe(name)
for (f_, blk), idx in sorted(B.blocks(rows, fn).items()):
    if sub not in blk:
        continue
    f = B.fit_block_fx(rows, idx, o, spec)
    k, n = rows[idx[0]]["k"], rows[idx[0]]["n"]
    C = B.R3._ld_of(mpmath.mpf(B.Fraction(*LDB(f["C0"]).as_integer_ratio()).numerator)
                    / B.Fraction(*LDB(f["C0"]).as_integer_ratio()).denominator + mpmath.mpf(f["off_all"]) * 2**-53)
    cex = B.stirlerr_exact(n) - B.stirlerr_exact(k) - B.stirlerr_exact(n - k)
    cR = B._s(B.stirlerr(n, "R") - B.stirlerr(k, "R"), "e") - B.stirlerr(n - k, "R")
    u = lambda v: float((mpmath.mpf(B.Fraction(*LDB(v).as_integer_ratio()).numerator)  # noqa: E731
                         / B.Fraction(*LDB(v).as_integer_ratio()).denominator - cex) * 2**53)
    print(f"{blk:24s} {f['br']}  all {f['all']}/{f['n']}  hold {f['hold']}/{f['nod']}  C-exact {u(C):+8.2f}  R-exact {u(cR):+8.2f}")
