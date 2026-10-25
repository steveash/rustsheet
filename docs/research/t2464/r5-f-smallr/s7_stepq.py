"""Step-quantum test: for candidate stored power terms Q (as they enter ln F), V_k = ln(Excel_k x_k) - Q_k; prints the
steps of V and the fraction of cells where V mod g (g = the grid of the remaining stored sum) is within 3 units of
its run mode.  Usage: s7_stepq.py SET tag"""
import sys

import mpmath
import numpy as np

import r5lib as R

LD, LDB, st = R.LD, R.LDB, R.st


def cands(x, m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    Lx = R.F.LN(x)
    mx = st(LDB(m) * LDB(x))
    out = {}
    for nm, c in (("a", a), ("b", b), ("h", h), ("a-1", a - 1), ("b+1", b + 1)):
        out[f"+fl({nm}LNx)"] = mpmath.mpf(st(LDB(c) * Lx))
        out[f"-fl({nm}LNx)"] = -mpmath.mpf(st(LDB(c) * Lx))
    out["-fl(bLNx)-LNx"] = -mpmath.mpf(st(LDB(b) * Lx)) - mpmath.mpf(float(Lx))
    out["+fl(aLNmx)"] = mpmath.mpf(st(LDB(a) * R.F.LN(mx)))
    out["-fl(bLNmx)"] = -mpmath.mpf(st(LDB(b) * R.F.LN(mx)))
    out["-fl(hLNmx)"] = -mpmath.mpf(st(LDB(h) * R.F.LN(mx)))
    r1 = st(LDB(n) / LDB(mx))
    out["+fl(bLNr)"] = mpmath.mpf(st(LDB(b) * R.F.LN(r1)))
    return out


src, tag = sys.argv[1], sys.argv[2]
for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
    print(blk)
    tau = [mpmath.log(mpmath.mpf(q["excel"])) + mpmath.log(mpmath.mpf(q["x"])) for q in rs]
    cs = [cands(q["x"], m, n) for q in rs]
    for nm in cs[0]:
        V = [t - c[nm] for t, c in zip(tau, cs)]
        g = R.ulpu(float(V[0]))
        Vu = np.array([float((v - V[0]) * R.U) for v in V])
        md = np.mod(Vu + 1000 * g, g)
        md = np.minimum(md, g - md)
        dv = np.diff(Vu)
        print(f"  {nm:16s} V={float(V[0]):9.3f} g={g:5.0f} on-grid {np.mean(md <= 3):.2f} steps:",
              " ".join(f"{v:.0f}" for v in dv[:24]))
