"""a9: enumerate compositions of the shape constant C = L5 + Cs - lf/2 and score them on clean shapes.

Clean shapes: within-shape sd < 0.8 units and >= 16 cells (sweep + diverse; X model at the floor). The needed
residual is the needed offset minus the exact Loader constant at Excel's k, n, nk (exact stirlerr, exact logs).
A composition predicts its own deviation from that exact constant. Pieces: P = ln 2 pi (the double M_LN_2PI),
logs LN (Excel's, stored) or L64 (FYL2X left extended), stirlerr exact (table / accurate). Orders of lf:
  A ((P + lk) + lnk) - ln    B (P + lk) + (lnk - ln)    C P + (lk + (lnk - ln))    D (P + (lk + lnk)) - ln
  F1 (P + lk) + LN(fl(1 - k/n)) (R >= 2.7.2 LF1; Excel's log1p switches to LN(fl(1-t)) above 0.375)
  G P + LN(fl(fl(k nk) / n))      H LN(fl(fl(fl(2pi k) nk) / n))   (log of the product, R 2.0 style)
each op stored ('x') or extended ('e'); then the E-chain: 'ext' (L5 + Cs - lf/2 all extended) or 'K' (h = fl(L5 -
lf/2) stored, then + Cs) or 'S' (lf/2 stored first).
Usage: a9_compose.py [--top N] [--table-only] [--files sweep,diverse,grid]
"""

from __future__ import annotations

import csv
import itertools
import math
import sys

import mpmath

import a3_const as A3
import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
D, R64 = R.L.D, R.L.R64
P = mpf(A3.H.M_LN_2PI)
TWO_PI = mpf(A3.H.M_2PI)


def rnd(v, m):
    return mpf(D(v)) if m == "x" else R64(v)


def lns(v, lm):
    return mpf(R.L.LN(float(v))) if lm == "LN" else R.L.ln64(mpf(v))


def lf_comp(k, n, nk, order, ms, lm):
    lk, lnk, ln_ = lns(k, lm), lns(nk, lm), lns(n, lm)
    m1, m2, m3 = ms
    if order == "A":
        return rnd(rnd(rnd(P + lk, m1) + lnk, m2) - ln_, m3)
    if order == "B":
        return rnd(rnd(P + lk, m1) + rnd(lnk - ln_, m2), m3)
    if order == "C":
        return rnd(P + rnd(lk + rnd(lnk - ln_, m1), m2), m3)
    if order == "D":
        return rnd(rnd(P + rnd(lk + lnk, m1), m2) - ln_, m3)
    if order == "F1":
        return rnd(rnd(P + lk, m1) + lns(rnd(1 - rnd(k / n, m2), "x"), lm), m3)
    if order == "G":
        return rnd(P + lns(rnd(rnd(k * nk, m1) / n, "x"), lm), m3)
    return lns(rnd(rnd(rnd(TWO_PI * k, m1) * nk, m2) / n, "x"), lm)  # H


def clean_shapes(table_only=False, files=("sweep", "diverse")):
    out = []
    for w in files:
        for r in csv.DictReader(open(R.HERE / f"off_{w}_R.tsv"), delimiter="\t"):
            if float(r["within_sd"]) >= 0.8 or int(r["cells"]) < (8 if w == "grid" else 16):
                continue
            a, b = float(r["a"]), float(r["b"])
            k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
            if table_only and not all((2 * v == int(2 * v) and v <= 15) or v > 15 for v in (k, n, nk)):
                continue
            ex = ("exact", "exact")
            Cs = A3.stirlerr(n, ex) - A3.stirlerr(k, ex) - A3.stirlerr(nk, ex)
            cexL = mpmath.log(n + 1) + Cs - (mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(nk) - mpmath.log(n)) / 2
            need = float(r["offset_units"]) - float((cexL - R.exact_const(a, b)) / U)
            out.append(dict(a=a, b=b, k=k, n=n, nk=nk, Cs=Cs, cexL=cexL, need=need))
    return out


def predict(s, order, ms, lm, l5m, chain):
    lf = lf_comp(s["k"], s["n"], s["nk"], order, ms, lm)
    L5 = lns(s["n"] + 1, l5m)
    if chain == "K":
        C = mpf(D(L5 - lf / 2)) + s["Cs"]
    elif chain == "S":
        C = L5 + s["Cs"] - mpf(D(lf / 2))
    elif chain == "KC":  # (L5 + Cs) stored, then - lf/2
        C = mpf(D(L5 + s["Cs"])) - lf / 2
    else:
        C = L5 + s["Cs"] - lf / 2
    return float((C - s["cexL"]) / U)


def main():
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 15
    files = sys.argv[sys.argv.index("--files") + 1].split(",") if "--files" in sys.argv else ("sweep", "diverse")
    shapes = clean_shapes("--table-only" in sys.argv, files)
    print(f"{len(shapes)} clean shapes; needed residual vs exact Loader: rms "
          f"{math.sqrt(sum(s['need'] ** 2 for s in shapes) / len(shapes)):.2f}")
    res = []
    for order in ("A", "B", "C", "D", "F1", "G", "H"):
        for ms in itertools.product("xe", repeat=3):
            for lm in ("LN", "L64"):
                for l5m in ("LN", "L64"):
                    for chain in ("ext", "K", "S", "KC"):
                        d = [s["need"] - predict(s, order, ms, lm, l5m, chain) for s in shapes]
                        rms = math.sqrt(sum(v * v for v in d) / len(d))
                        res.append((rms, sum(abs(v) <= 1 for v in d), order, "".join(ms), lm, l5m, chain, sum(d) / len(d)))
    res.sort()
    for rms, c1, *cfg, mean in res[:top]:
        print(f"  rms {rms:5.2f} |d|<=1 {c1:4d}/{len(shapes)} mean {mean:+.2f}  lf={cfg[0]} ops={cfg[1]} logs={cfg[2]} L5={cfg[3]} chain={cfg[4]}")


if __name__ == "__main__":
    main()
