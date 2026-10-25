"""b6: the x = 1 cells only (P = 0, no 1/x error: F = EXP(E)). Excel's admissible E (absolute) per shape, and
candidate constructions of ln F(1) = -1/2 L(r) + 1/2 ln(w/2pi) + st(h) - st(w) - st(o), w = min shape, o = max,
r = fl(w/o) (= Excel's r at x = 1). Prints hits and the signed miss (units) per candidate.
Usage: b6_x1.py SET [cand ...]"""

from __future__ import annotations

import sys

import numpy as np

import blib as B

C, SL, SD, F, mp, LDB, st = B.C, B.SL, B.SD, B.F, B.mp, B.LDB, B.st
U = B.U
ex = C.ex
TWO_PI = 2 * float(mp.pi)


def LN(v):
    return SL.LN(v)


def stx(v):
    return F.stirlerr_exact(v)


def target(r):
    iv = C.e_interval(r)
    if iv is None:
        return None
    E0 = ex(C.E0_of(r))
    return E0 + mp.mpf(iv[0]) / U, E0 + mp.mpf(iv[1]) / U


def shape(m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    side = 1 if m <= n else 2
    return a, b, h, side, min(a, b), max(a, b)


def cand(name, m, n):
    a, b, h, side, w, o = shape(m, n)
    r = st(LDB(w) / LDB(o))
    Lr = SL.L_xl(r)
    H = -LDB(h) * Lr
    K = C.K_exact(m, n, side)
    dst = stx(h) - stx(a) - stx(b)
    if name == "mxd":  # exact K + H
        return K + ex(H)
    if name == "msd":
        return C.K_sd(m, n, side) + ex(H)
    if name == "hk":  # fl(fl(K) + H) + (K - fl(K))
        Kh = float(K)
        return ex(st(LDB(Kh) + H)) + (K - mp.mpf(Kh))
    if name == "hksd":
        Ks = C.K_sd(m, n, side)
        Kh = float(Ks)
        return ex(st(LDB(Kh) + H)) + (Ks - mp.mpf(Kh))
    if name == "Est":  # E stored as a double, exact K
        return ex(st(SL.ld(K) + H))
    # Loader-style direct forms with exact stirlerr: E = -1/2 L + 1/2 ln(w/2pi) + dst
    half_lnw2pi = mp.log(mp.mpf(w) / (2 * mp.pi)) / 2
    if name == "ld_exactln":
        return -ex(Lr) / 2 + half_lnw2pi + dst
    if name == "ld_LN":  # 1/2 (LN(w) - LN(2pi)) extended
        return -ex(Lr) / 2 + (ex(LN(w)) - ex(LN(TWO_PI))) / 2 + dst
    if name == "ld_LNq":  # 1/2 LN(fl(w/2pi))
        return -ex(Lr) / 2 + ex(LN(st(LDB(w) / LDB(TWO_PI)))) / 2 + dst
    if name == "ld_LNq2":  # 1/2 LN(fl(w/(2pi s)))  (s = fl(1 + r))
        s = st(LDB(1) + LDB(r))
        return ex(LN(st(LDB(st(LDB(w) / LDB(TWO_PI))) / LDB(s)))) / 2 + dst
    if name == "ld_LNall":  # -1/2 LN(fl(2 pi fl(1+r) / w))
        s = st(LDB(1) + LDB(r))
        return -ex(LN(st(LDB(st(LDB(TWO_PI) * LDB(s))) / LDB(w)))) / 2 + dst
    if name == "ld_Lr":  # exact log1p at the double r
        return -mp.log1p(ex(r)) / 2 + half_lnw2pi + dst
    if name == "exact":
        return -mp.log1p(mp.mpf(w) / o) / 2 + half_lnw2pi + dst
    if name.startswith("st_"):  # stored-piece family on exact pieces: A = -1/2 log1p(w/o), B = 1/2 ln(w/2pi), D = dst
        A = -mp.log1p(mp.mpf(w) / o) / 2
        Bv, D = half_lnw2pi, dst
        f = lambda v: mp.mpf(float(v))  # noqa: E731
        return {"st_E": f(A + Bv + D), "st_AB_D": f(f(A + Bv) + D), "st_A_B_D": f(A) + f(Bv) + f(D),
                "st_AB": f(A + Bv) + D, "st_A_BD": f(A) + f(Bv + D), "st_ABDs": f(f(f(A) + f(Bv)) + f(D)),
                "st_A": f(A) + Bv + D, "st_B": A + f(Bv) + D, "st_BD": A + f(Bv + D),
                "st_hw": A + f(mp.log(w) / 2) - f(mp.log(2 * mp.pi) / 2) + D,
                "st_hw2": A + f(f(mp.log(w) / 2) - f(mp.log(2 * mp.pi) / 2)) + D}[name]
    if name.startswith("ld_exactln+"):  # offset scan (units)
        return -ex(Lr) / 2 + half_lnw2pi + dst + mp.mpf(name.split("+")[1]) / U
    raise ValueError(name)


CANDS = ["st_E", "st_AB_D", "st_A_B_D", "st_AB", "st_A_BD", "st_ABDs", "st_A", "st_B", "st_BD", "st_hw", "st_hw2", "exact", "ld_Lr", "mxd", "msd", "hk", "hksd", "Est", "ld_exactln", "ld_LN", "ld_LNq", "ld_LNq2", "ld_LNall"]


def score(rows, names):
    x1 = [r for r in rows if r["x"] == 1.0]
    tg = [(r, target(r)) for r in x1]
    tg = [(r, t) for r, t in tg if t is not None]
    out = {}
    for nm in names:
        miss = []
        for r, (lo, hi) in tg:
            E = cand(nm, r["d1"], r["d2"])
            miss.append(0.0 if lo <= E <= hi else float((E - (lo if E < lo else hi)) * U))
        out[nm] = np.array(miss)
    return tg, out


if __name__ == "__main__":
    names = sys.argv[2:] or CANDS
    tg, out = score(B.load(sys.argv[1]), names)
    for nm, ms in out.items():
        print(f"{nm:12s} hits {int(np.sum(ms == 0)):4d}/{len(ms)}  |miss|<1 {int(np.sum(abs(ms) < 1))}  rms {np.sqrt(np.mean(ms**2)):.2f} mean {ms.mean():+.2f}")
