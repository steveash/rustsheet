"""Needed per-cell shift of the constant: best free-C fit of CHAIN, then for every cell the set of shifts j (units,
step 0.25 over +-W) of C that make that cell match (|res| <= TOL with the run's offset fixed).  Prints per cell the
admissible [lo, hi] (the longest run of admissible j containing the one closest to 0) and candidate regressors:
  er  = h (r - r_exact)/(1 + r)  (units; r's rounding as it enters -h log1p(r))
  eLx = Excel LN(x) - ln x (units)        eP = P - c LN(x) (units, P's own rounding)
Usage: s10_need.py SET tag chain [W]"""
import sys

import mpmath
import numpy as np

import r5lib as CL
import r5lib as R

LD = R.LD
W = 64.0


def need(rs, m, n, ch):
    sc, d, res = CL.scan(rs, m, n, ch, ret=True)
    toks, arr, meas, K0 = CL.prepare(rs, m, n, ch)
    Y0 = meas[0]
    Y = np.array([float((v - Y0) * R.U) for v in meas])
    Kh = LD(float(K0))
    js = np.arange(-W, W + 0.01, 0.25)
    C = (Kh + (LD(d) + js.astype(LD)) * LD(2.0) ** -53)[:, None]
    Ym = R.units_rel(CL.evaluate(toks, arr, C), Y0)
    rr = Y[None, :] - Ym
    off = np.median(Y - Ym[np.argmin(np.abs(js))])
    ok = np.abs(rr - off) <= CL.TOL
    out = []
    for k in range(len(rs)):
        idx = np.nonzero(ok[:, k])[0]
        if len(idx) == 0:
            out.append((np.nan, np.nan))
            continue
        i0 = idx[np.argmin(np.abs(js[idx]))]
        lo = hi = i0
        while lo - 1 >= 0 and ok[lo - 1, k]:
            lo -= 1
        while hi + 1 < len(js) and ok[hi + 1, k]:
            hi += 1
        out.append((js[lo], js[hi]))
    return sc, out


def regressors(rs, m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    out = []
    for q in rs:
        x = q["x"]
        side1, _, _, _, r, P = R.base(x, m, n)
        X = mpmath.mpf(x)
        rex = m * X / n if side1 else n / (m * X)
        er = float(h * (mpmath.mpf(r) - rex) / (1 + rex) * R.U)
        Lx = mpmath.mpf(float(R.F.LN(x)))
        eLx = float((Lx - mpmath.log(X)) * R.U)
        c = a if side1 else -b
        eP = float((P - c * Lx) * R.U)
        out.append((er, eLx, eP))
    return out


if __name__ == "__main__":
    src, tag, ch = sys.argv[1:4]
    for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
        sc, iv = need(rs, m, n, ch)
        rg = regressors(rs, m, n)
        print(blk, ch, sc, len(rs))
        for k, ((lo, hi), (er, eLx, eP)) in enumerate(zip(iv, rg)):
            mark = "*" if not (lo <= 0 <= hi) else " "
            print(f"{k:3d}{mark} need [{lo:+6.2f},{hi:+6.2f}] er {er:+6.1f} eLx {eLx:+5.2f} eP {eP:+6.1f}")
