"""Constant constructions against the measured per-(shape, side) offsets of Excel's K (s4_kscan.out: free-delta
best interval on top of M-xd, groups with a best count >= MINC). delta_cand = (K_cand - K_exact) in units.
Prints RMS and the number within +-1 and +-2 units per candidate.  Usage: s17_kfit.py [MINC] [--show cand]"""
import itertools
import re
import sys

import numpy as np

import slib as L

LDB, st, mp = L.LDB, L.st, L.mpmath
args = [a for a in sys.argv[1:] if not a.startswith("--")]
MINC = int(args[0]) if args else 25
show = sys.argv[sys.argv.index("--show") + 1] if "--show" in sys.argv else None
obs = []
for line in open(L.HERE / "s4_kscan.out"):
    m = re.match(r"\((\d+),(\d+)\) s(\d) n\s+(\d+) best\s+(\d+) delta \[\s*([-+.\d]+),\s*([-+.\d]+)\]", line)
    if m and int(m[5]) >= MINC:
        obs.append((float(m[1]), float(m[2]), int(m[3]), 0.5 * (float(m[6]) + float(m[7]))))
TK = L.B.BX.Kernel(L.B.BX.TOMS_XL)
TT = L.B.BX.Kernel(L.B.BX.R_LIKE)
vals = sorted({v for m, n, _, _ in obs for v in (m / 2, n / 2, (m + n) / 2)})
L.xl("GAMMALN", vals)
L.xl("GAMMA", vals)


def fr(v):
    p, q = LDB(v).as_integer_ratio()
    return L.mpf(p) / q


def lnB_forms(a, b):
    h = a + b
    g = {v: L.mpf(L.GL(v)) for v in (a, b, h)}
    return {
        "exact": L.lnB_exact(2 * a, 2 * b),
        "gl(a+b)-h": L.mpf(st(LDB(st(LDB(g[a]) + LDB(g[b]))) - LDB(g[h]))),
        "gl a+(b-h)": L.mpf(st(LDB(g[a]) + LDB(st(LDB(g[b]) - LDB(g[h]))))),
        "gl ext": g[a] + g[b] - g[h],
        "toms_xl": L.mpf(TK.betaln(a, b)),
        "toms": L.mpf(TT.betaln(a, b)),
        "gamma": L.mpf(float(LDB(L.LN(st(LDB(L.GA(a)) * LDB(L.GA(b)) / LDB(L.GA(h))))))),
    }


def ratio_forms(m, n, side):
    a, b, h = m / 2, n / 2, (m + n) / 2
    lm, ln_ = fr(L.LN(m)), fr(L.LN(n))
    c = a if side == 1 else -b
    out = {
        "exact": c * mp.log(L.mpf(m) / n),
        "fl(c LN(m/n))": L.mpf(st(LDB(c) * L.LN(st(LDB(m) / LDB(n))))),
        "c LN(m/n) ext": fr(LDB(c) * L.LN(st(LDB(m) / LDB(n)))),
        "fl(c(LNm-LNn))": L.mpf(st(LDB(c) * (L.LN(m) - L.LN(n)))),
        "fl(cLNm)-fl(cLNn)": L.mpf(st(LDB(c) * L.LN(m))) - L.mpf(st(LDB(c) * L.LN(n))),
    }
    big = h * (ln_ if side == 1 else lm)
    out["amln+bnln-h"] = L.mpf(st(LDB(a) * L.LN(m))) + L.mpf(st(LDB(b) * L.LN(n))) - L.mpf(st(LDB(h) * L.LN(n if side == 1 else m)))
    out["(a,b,h) ext"] = a * lm + b * ln_ - big
    out["fl(fl(am+bn)-h)"] = L.mpf(st(LDB(st(LDB(st(LDB(a) * L.LN(m))) + LDB(st(LDB(b) * L.LN(n))))) - LDB(st(LDB(h) * L.LN(n if side == 1 else m)))))
    return out


res: dict = {}
for m, n, side, d in obs:
    K = L.K_exact(m, n, side)
    for (kb, B), (kr, R) in itertools.product(lnB_forms(m / 2, n / 2).items(), ratio_forms(m, n, side).items()):
        dc = float((-B + R - K) / L.U)
        res.setdefault((kb, kr), []).append((dc, d, m, n, side))
rank = []
for k, v in res.items():
    e = np.array([a - b for a, b, *_ in v])
    rank.append((float(np.sqrt((e**2).mean())), int((abs(e) <= 1).sum()), int((abs(e) <= 2).sum()), k))
print(f"{len(obs)} groups (best count >= {MINC}); null (all delta 0): RMS "
      f"{float(np.sqrt(np.mean([d * d for *_, d in obs]))):.2f}")
for r in sorted(rank)[:20]:
    print(f"RMS {r[0]:6.2f} within1 {r[1]:3d} within2 {r[2]:3d}  {r[3]}")
if show:
    kb, kr = show.split("|")
    for dc, d, m, n, s in res[(kb, kr)]:
        print(f"({m:g},{n:g}) s{s} obs {d:+6.2f} cand {dc:+6.2f}")
