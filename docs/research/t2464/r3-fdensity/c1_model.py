"""c1: the ratio model of Excel's F density (b4-b6).  With mx = fl(m x) and h = (m + n)/2:
  mx <= n:  r = fl(mx/n),  lnF = K + (a - 1) ln x + a ln(m/n) - h L(r)
  mx >  n:  r = fl(n/mx),  lnF = K - (b + 1) ln x - b ln(m/n) - h L(r)
  'rpow' variants: lnF = ln(m/n) - lnB(a, b) + fl(c LN(r)) - h L(r), c = a - 1 (mx <= n) or b + 1 (mx > n)
  K = lnGamma(h) - lnGamma(a) - lnGamma(b) (accurate: Excel's x = 1 sweeps carry no GAMMALN error, b1_sweep.py)
  L(r) = Excel's ln(1 + r): 'xl' the atanh series for r <= 0.375 and LN(fl(1 + r)) above; 'ln' always LN(fl(1+r));
         'm' exact ln(1 + r) of the stored r.
Variants: ln x term as Excel LN with the product stored ('x') or exact ('m'); h L stored ('x') or not ('e').
The constant parts are exact.  E goes into EXP V6 (extended).
Usage: c1_model.py SRC [variant ...]   SRC: probe:a | probe:hold | cells | sw | holdsw"""
import collections
import functools
import sys

import mpmath

import flib as F

LDB, st = F.LDB, F.st
mpf = mpmath.mpf

VARIANTS = {
    "CR": None,
    "ratio-xl": dict(L="xl", lx="x", hl="x"),
    "ratio-xl-e": dict(L="xl", lx="x", hl="e"),
    "ratio-ln": dict(L="ln", lx="x", hl="x"),
    "ratio-m": dict(L="m", lx="m", hl="e"),
    "ratio-xl-lxm": dict(L="xl", lx="m", hl="e"),
    "rpow-xl": dict(L="xl", lx="r", hl="x"),
    "rpow-xl-e": dict(L="xl", lx="r", hl="x", acc="e"),
    "rpow-xl-he": dict(L="xl", lx="r", hl="e"),
    "rpow-ln": dict(L="ln", lx="r", hl="x"),
}


@functools.lru_cache(maxsize=None)
def konst(m, n, side):
    a, b, h = mpf(m) / 2, mpf(n) / 2, (mpf(m) + mpf(n)) / 2
    K = mpmath.loggamma(h) - mpmath.loggamma(a) - mpmath.loggamma(b)
    return K + (a if side == 1 else -b) * mpmath.log(mpf(m) / mpf(n))


@functools.lru_cache(maxsize=None)
def konst_r(m, n):
    a, b = mpf(m) / 2, mpf(n) / 2
    return mpmath.log(mpf(m) / mpf(n)) - (mpmath.loggamma(a) + mpmath.loggamma(b) - mpmath.loggamma(a + b))


def L_of(r, mode):
    if mode == "m":
        return mpmath.log1p(mpf(r))
    if mode == "xl" and r <= 0.375:
        return mpf(F.TD.log1p_xl(r))
    return mpf(float(F.LN(st(1 + LDB(r)))))


def value(x, m, n, o):
    a, b, h = m / 2, n / 2, (m + n) / 2
    mx = st(LDB(m) * LDB(x))
    if mx <= n:
        side, r, cx = 1, st(LDB(mx) / LDB(n)), a - 1
    else:
        side, r, cx = 2, st(LDB(n) / LDB(mx)), -(b + 1)
    l = L_of(r, o["L"])
    hl = h * l if o["hl"] == "e" else mpf(st(LDB(h) * LDB(float(l))))
    if o["lx"] == "r":  # F = (m/n) r^c (1 + r)^-h / B(a, b): c = a - 1 (r = mx/n) or b + 1 (r = n/mx)
        c = a - 1 if side == 1 else b + 1
        tx = mpf(st(LDB(c) * F.LN(r)))
        E = konst_r(m, n) + tx - hl
        return F.R3.exp_v6(F.R3._ld_of(E))
    if o["lx"] == "m":
        tx = cx * mpmath.log(mpf(x))
    else:
        tx = mpf(st(LDB(cx) * F.LN(x)))
    E = konst(m, n, side) + tx - hl
    return F.R3.exp_v6(F.R3._ld_of(E))


def load(src):
    if src == "cells":
        return F.with_exact(F.cells_rows())
    import mkprobe

    return F.with_exact(mkprobe.rows(src.split(":", 1)[-1]))


def main():
    src = sys.argv[1]
    names = sys.argv[2:] or list(VARIANTS)
    rows = load(src)
    for nm in names:
        o = VARIANTS[nm]
        g = collections.defaultdict(list)
        for r in rows:
            v = r["cr"] if o is None else value(r["x"], r["d1"], r["d2"], o)
            m, n = r["d1"], r["d2"]
            cls = "min(d)<=2" if min(m, n) <= 2 else ("max(d)>=50" if max(m, n) >= 50 else "other")
            g["all"].append((v, r["excel"]))
            g[cls].append((v, r["excel"]))
        print(f"{src} {nm:13s} " + " | ".join(f"{k} {F.fmt(F.tally(v))}" for k, v in sorted(g.items())))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
