"""Fixed C (stirlerr, R order) and lc = lc_chain(C, b1, b2); scan the factor F = F0 (1 + d 2^-53), F0 = exact
sqrt(n/(2 pi k (n-k))), for out = fl(EXP_ext(lc) * F): exact count vs d, and where the formula Fs fall.
Usage: b18_fscan2.py <probe> <fn> <block> [lc spec] [stir] [cs] [span] [step]"""
import sys
from fractions import Fraction

import mpmath
import numpy as np

import blib as B

LDB = B.LDB
name, fn, blk = sys.argv[1:4]
spec = sys.argv[4] if len(sys.argv) > 4 else "R:ex"
sm = sys.argv[5] if len(sys.argv) > 5 else "R"
csm = sys.argv[6] if len(sys.argv) > 6 else "e"
span = float(sys.argv[7]) if len(sys.argv) > 7 else 6
step = float(sys.argv[8]) if len(sys.argv) > 8 else 0.0625
rows = [r for r in B.load_probe(name) if r["fn"] == fn and r["blk"] == blk]
k, n = rows[0]["k"], rows[0]["n"]
o = dict(mr="xx")
X = [B.xpart_v(r["p"], k, n, o) for r in rows]
C = B._s(B._s(B.stirlerr(n, sm) - B.stirlerr(k, sm), csm) - B.stirlerr(n - k, sm), csm)
F0 = mpmath.sqrt(mpmath.mpf(n) / (2 * mpmath.pi * k * (n - k)))
es = [B.R3.exp_v6(B.lc_chain(spec, C, b1, b2), True) for b1, b2, _ in X]
out = []
for d in np.arange(-span, span + 1e-9, step):
    F = B.R3._ld_of(F0 * (1 + mpmath.mpf(d) * 2**-53))
    hit = sum(B.st(e * F) == r["excel"] for e, r in zip(es, rows))
    out.append((d, hit))
mx = max(h for _, h in out)
print(blk, spec, "max", mx, "/", len(rows), " d at max:", [f"{d:+.3f}" for d, h in out if h == mx][:3], "...",
      [f"{d:+.3f}" for d, h in out if h == mx][-3:])
for fl in ("eeee", "xxxx", "xxxe", "xxex", "exxx", "eeex"):
    F = B.fx_F(k, n, dict(F=fl))
    d = float((mpmath.mpf(Fraction(*LDB(F).as_integer_ratio()).numerator) / Fraction(*LDB(F).as_integer_ratio()).denominator / F0 - 1) * 2**53)
    hit = sum(B.st(e * F) == r["excel"] for e, r in zip(es, rows))
    print(f"   F[{fl}] d = {d:+.3f}  hits {hit}")
