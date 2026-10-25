"""a10: on the half-integer grid probe, fit per-argument stirlerr corrections e(v) and LN-rounding coefficients.

residual(shape) = needed offset - exact Loader constant at Excel's k, n, nk  (units 2^-53)
model: residual = e(n) - e(k) - e(nk) + sum_j beta_j f_j + c,  e(v) one unknown per half-integer v (v <= 30),
f_j = rounding errors of LN(n+1) (+1), LN(k), LN(nk) (-1/2), LN(n) (+1/2) as they enter E.
e(v) is identified up to e(v) + alpha v + gamma (n = k + nk on the grid); it is reported with e(0.5) = e(1) = 0.
Usage: a10_grid.py [--holdhalf]   (--holdhalf: fit on even-md5 shapes, report residual on the others)
"""

from __future__ import annotations

import csv
import hashlib
import sys

import mpmath
import numpy as np

import a7_creg as A7
import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53


def need(a, b, o):
    k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
    ex = ("exact", "exact")
    Cs = A7.A3.stirlerr(n, ex) - A7.A3.stirlerr(k, ex) - A7.A3.stirlerr(nk, ex)
    cexL = mpmath.log(n + 1) + Cs - (mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(nk) - mpmath.log(n)) / 2
    return o - float((cexL - R.exact_const(a, b)) / U), float(k), float(n), float(nk)


def main():
    rows = list(csv.DictReader(open(R.HERE / "off_grid_R.tsv"), delimiter="\t"))
    data = []
    for r in rows:
        if float(r["within_sd"]) > 1.0:
            continue
        a, b = float(r["a"]), float(r["b"])
        y, k, n, nk = need(a, b, float(r["offset_units"]))
        names, f = A7.feats(a, b)
        fl = [f[names.index(c)] for c in ("L5", "LNk", "LNnk", "LNn")]
        data.append((a, b, y, k, n, nk, fl))
    vs = sorted({v for d in data for v in d[3:6]})
    idx = {v: i for i, v in enumerate(vs)}
    nv = len(vs)
    hold = ["--holdhalf" in sys.argv and hashlib.md5(f"{d[0]}|{d[1]}".encode()).digest()[0] & 1 for d in data]
    F, Y = [], []
    for d in data:
        row = [0.0] * (nv + 5)
        row[idx[d[4]]] += 1
        row[idx[d[3]]] -= 1
        row[idx[d[5]]] -= 1
        row[nv : nv + 4] = d[6]
        row[nv + 4] = 1.0
        F.append(row)
        Y.append(d[2])
    F, Y = np.array(F), np.array(Y)
    fit = np.array([not h for h in hold])
    c, *_ = np.linalg.lstsq(F[fit], Y[fit], rcond=None)
    r = Y - F @ c
    print(f"{len(Y)} grid shapes (within-shape sd <= 1); residual sd {Y.std():.2f} -> fit {r[fit].std():.2f}"
          + (f", held-out half {r[~fit].std():.2f}" if (~fit).any() else ""))
    print("   LN coefs (L5, LNk, LNnk, LNn): " + ", ".join(f"{v:+.2f}" for v in c[nv : nv + 4]) + f"; const {c[nv + 4]:+.2f}")
    e = c[:nv]
    # gauge: remove alpha v + gamma so that e(0.5) = e(1) = 0
    v0, v1 = idx.get(0.5), idx.get(1.0)
    alpha = (e[v1] - e[v0]) / 0.5
    gamma = e[v0] - alpha * 0.5
    print("   e(v) units (gauge e(0.5) = e(1) = 0):")
    print("   " + "  ".join(f"{v:g}:{e[i] - alpha * v - gamma:+.1f}" for v, i in idx.items()))
    # the same without e(v): LN roundings and a constant only
    G = F[:, nv:]
    cg, *_ = np.linalg.lstsq(G[fit], Y[fit], rcond=None)
    rg = Y - G @ cg
    print(f"   without e(v): residual sd fit {rg[fit].std():.2f}" + (f", held-out {rg[~fit].std():.2f}" if (~fit).any() else "")
          + "; LN coefs " + ", ".join(f"{v:+.2f}" for v in cg[:4]))
    one = Y - G[:, :4] @ np.ones(4)
    print(f"   LN coefs fixed at 1: residual mean {one.mean():+.2f} sd {one.std():.2f}")
    ns = np.array([d[4] for d in data])
    for lo, hi in ((0, 8), (8, 15.01), (15.01, 22), (22, 31)):
        m = (ns >= lo) & (ns < hi)
        print(f"      n in [{lo:g}, {hi:g}): {m.sum():3d} shapes, mean {one[m].mean():+.2f} sd {one[m].std():.2f}")
    with open(R.HERE / "grid_e.tsv", "w") as fh:
        fh.write("v\te_units\n")
        for v, i in idx.items():
            fh.write(f"{v}\t{e[i] - alpha * v - gamma:+.3f}\n")


if __name__ == "__main__":
    main()
