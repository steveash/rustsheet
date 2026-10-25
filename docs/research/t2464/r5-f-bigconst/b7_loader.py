"""b7: value-space candidates for F(1; m, n) at the x = 1 cells (bit-exact hits): Loader's dbinom_raw(a; h, p)
times a b / (h x) (the unshifted identity f = a b / (h x) dbinom_raw(a, h, p, q)), R's shifted df, and log-space
variants, with Excel's EXP V6 / LN / log1p and stored doubles. stirlerr = the double nearest the exact value (Loader's
table for v <= 15 is exact-rounded; the series above it is accurate to < 0.02 units here).
Usage: b7_loader.py SET [cand ...]"""

from __future__ import annotations

import sys

import blib as B

C, SL, F, mp, LDB, st = B.C, B.SL, B.F, B.mp, B.LDB, B.st
EXP = SL.EXP
LN = SL.LN
L1P = SL.L_xl
LN2PI = float(mp.log(2 * mp.pi))  # R's M_LN_2PI as a double
HL2PI = float(mp.log(2 * mp.pi) / 2)  # M_LN_SQRT_2PI


def sx(v):
    return float(F.stirlerr_exact(v))


def S(*ops):
    """Left-to-right stored sum of doubles."""
    acc = LDB(ops[0])
    for o in ops[1:]:
        acc = LDB(st(acc + LDB(o)))
    return float(acc)


def cand(name, m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    p, q = st(LDB(a) / LDB(h)), st(LDB(b) / LDB(h))
    lc = S(sx(h), -sx(a), -sx(b))
    lc_ext = LDB(sx(h)) - LDB(sx(a)) - LDB(sx(b))
    lf_R = S(LN2PI, float(LN(a)), float(L1P(-st(LDB(a) / LDB(h)))))
    ab_h = st(LDB(st(LDB(a) * LDB(b))) / LDB(h))
    if name == "L_val":  # fl(EXP(lc - lf/2) * fl(ab/h))
        return st(LDB(EXP(st(LDB(lc) - LDB(0.5) * LDB(lf_R)))) * LDB(ab_h))
    if name == "L_valx":  # extended E, EXP kept extended, times ab/h
        return st(B.R3.exp_v6(lc_ext - LDB(0.5) * LDB(lf_R), True) * LDB(ab_h))
    if name == "L_log":  # EXP(lc - lf/2 + LN(ab/h)) extended
        return EXP(lc_ext - LDB(0.5) * LDB(lf_R) + LN(ab_h))
    if name == "L_sym":  # w = min: lf = LN 2pi + LN w + log1p(-w/h), times w o / h
        w, o = min(a, b), max(a, b)
        lf = S(LN2PI, float(LN(w)), float(L1P(-st(LDB(w) / LDB(h)))))
        return st(LDB(EXP(st(LDB(lc) - LDB(0.5) * LDB(lf)))) * LDB(ab_h))
    if name == "L_sqrt":  # fl(fl(ab/h) * EXP(lc) / SQRT(2 pi a q))
        den = st(LDB(st(LDB(2 * float(mp.pi)) * LDB(a))) * LDB(q))
        return st(LDB(ab_h) * LDB(EXP(lc)) / LDB(float(mp.sqrt(den))))
    raise ValueError(name)


CANDS = ["L_val", "L_valx", "L_log", "L_sym", "L_sqrt"]

if __name__ == "__main__":
    x1 = [r for r in B.load(sys.argv[1]) if r["x"] == 1.0]
    for nm in sys.argv[2:] or CANDS:
        try:
            vals = [cand(nm, r["d1"], r["d2"]) for r in x1]
        except AttributeError as e:
            print(nm, "skip", e)
            continue
        print(f"{nm:8s}", F.fmt(F.tally(list(zip(vals, [r["excel"] for r in x1])))), "| CR",
              F.fmt(F.tally([(F.cr(r["exact"]), r["excel"]) for r in x1])))
