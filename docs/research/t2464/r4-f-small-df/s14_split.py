"""Free-constant scan of split chains (two EXP factors, value-space constant, final 1/x placement) per (shape,
side) on probe mya. Pieces: K (exact + delta units), P = fl(c LN x) (c = a | -b), H = h L(r) (extended), X = x.
Variants (delta always rides on the factor holding K; 'C' = a double constant fl(exp(K)(1+delta)) in value space):
  m-xd     fl(EXP(K+P-H)/x)                    xn64     fl(EXP(K+P-H-ln64 x))
  s3       fl(fl(EXP(K+P)*EXP(-H))/x)          s4       fl(fl(EXP(K+P)/x)*EXP(-H))
  s5       fl(EXP(K+P-ln64 x)*EXP(-H))         s6       fl(EXP(K-H)*fl(EXP(P)/x))
  s7       fl(EXP(K-H)*EXP(P-ln64 x))          s8       fl(fl(EXP(K-H)*EXP(P))/x)
  c1       fl(C*fl(EXP(P-H)/x))                c2       fl(C*EXP(P-H-ln64 x))
  c3       fl(fl(C*EXP(P-H))/x)                c4       fl(C*fl(EXP(P)/x)*EXP(-H)) (left to right)
Usage: s14_split.py maxd [variant ...]"""
import sys

import numpy as np

import slib as L

LDB, st = L.LDB, L.st
maxd = float(sys.argv[1])
want = sys.argv[2:]
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]
grid = np.arange(-12, 12.0001, 0.05).astype(LDB)
UL = LDB(2.0) ** -53
EV = L.exp_v6_vec


def F64(v):
    return np.asarray(v, dtype=LDB).astype(np.float64).astype(LDB)


def variants(K, P, H, l64, X, Xd):
    """Each returns an array over the delta grid."""
    Kd = K + grid * UL
    eP, eH = LDB(L.EXP(P)), LDB(L.EXP(-H))
    ePl = LDB(L.EXP(P - l64))
    ePH, ePHl = LDB(L.EXP(P - H)), LDB(L.EXP(P - H - l64))
    C = F64(np.array([float(L.mpmath.exp(L.mpf(float(K))) * (1 + float(d) * 2**-53)) for d in grid]))  # noqa
    return {
        "m-xd": F64(F64(EV(Kd + P - H)) / X),
        "xn64": F64(EV(Kd + P - H - l64)),
        "m-xd-st": F64(F64(EV(F64(Kd + P - H))) / X),
        "xn64-st": F64(EV(F64(Kd + P - H - l64))),
        "xn-st": F64(EV(F64(F64(Kd + P - H) - LDB(L.LN(Xd))))),
        "m-xd-st2": F64(F64(EV(F64(F64(Kd + P) - H))) / X),
        "m-xd-stH": F64(F64(EV(Kd + P - F64(H))) / X),
        "s3": F64(F64(F64(EV(Kd + P)) * eH) / X),
        "s4": F64(F64(F64(EV(Kd + P)) / X) * eH),
        "s5": F64(F64(EV(Kd + P - l64)) * eH),
        "s6": F64(F64(EV(Kd - H)) * F64(eP / X)),
        "s7": F64(F64(EV(Kd - H)) * ePl),
        "s8": F64(F64(F64(EV(Kd - H)) * eP) / X),
        "c1": F64(C * F64(ePH / X)),
        "c2": F64(C * ePHl),
        "c3": F64(F64(C * ePH) / X),
        "c4": F64(F64(C * F64(eP / X)) * eH),
    }


g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s), []).append(r)
tot: dict = {}
for (m, n, s), rs in sorted(g.items()):
    cnt: dict = {}
    for r in rs:
        x = r["x"]
        a, b, h = m / 2, n / 2, (m + n) / 2
        _, rr = L.side_r(x, m, n)
        c = a if s == 1 else -b
        P = LDB(st(LDB(c) * L.LN(x)))
        H = LDB(h) * L.L_xl(rr)
        _, Kx, _ = L.chain(x, m, n, "xd")
        K = L.ld(Kx)
        for k, v in variants(K, P, H, L.R3.LN64(x), LDB(x), x).items():
            if want and k not in want:
                continue
            cnt.setdefault(k, np.zeros(len(grid), int))
            cnt[k] += v.astype(np.float64) == r["excel"]
    line = []
    for k, v in cnt.items():
        tot[k] = tot.get(k, 0) + int(v.max())
        line.append(f"{k} {int(v.max()):2d}")
    print(f"({m:g},{n:g}) s{s} {len(rs)} | " + " ".join(line), flush=True)
print("TOTAL", tot)
