"""s3: partial-sum orders.  Terms (signed as in lnF): K' (= lnGamma(h)-lnGamma(a)-lnGamma(b) + a ln(m/n) | - b ln(m/n),
stored as the nearest double), P = fl(c*LN(x)) (c = a | b, sign + | -), H = h*L(r) (stored 'x' or extended 'e'),
X = LN(x) (sign -).  For each order of the four terms, the running sums are x87 stored; their own roundings
rho1..rho3 join rx, eLx, eL as regressors.  Printed per order: median residual sd over the runs and the median
coefficients.  Usage: s3_order.py SRC MINMAX HMODE"""
import collections
import functools
import itertools
import sys

import mpmath
import numpy as np

import cands as C
import plib as P
import s2_reg

LDB, st, S, ex = C.LDB, C.st, C.S, C.ex
mpf = mpmath.mpf
U = P.U


@functools.lru_cache(maxsize=None)
def kprime(m, n, side1):
    a, b, h = mpf(m) / 2, mpf(n) / 2, (mpf(m) + mpf(n)) / 2
    K = mpmath.loggamma(h) - mpmath.loggamma(a) - mpmath.loggamma(b)
    return K + (a if side1 else -b) * mpmath.log(mpf(m) / mpf(n))


def terms(x, m, n, hmode):
    s, e = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c = a if side1 else b
    sg = 1 if side1 else -1
    Kd = LDB(float(kprime(m, n, side1)))
    Pt = S(LDB(c) * s["Lx"]) * sg
    L = s["L1p2"] if side1 else s["L1p1"]
    H = LDB(h) * L
    if hmode == "x":
        H = S(H)
    return {"K": Kd, "P": Pt, "H": -H, "X": -s["Lx"]}


def rhos(t, order):
    acc = t[order[0]]
    out = []
    for k in order[1:]:
        ext = acc + t[k]
        acc = S(ext)
        out.append(float((ex(acc) - ex(ext)) * U))
    return out


def main():
    src, minmax, hmode = sys.argv[1], float(sys.argv[2]), sys.argv[3]
    rows = P.load(src)
    g = collections.defaultdict(list)
    for r in rows:
        g[r["blk"]].append(r)
    blks = [(b, rs) for b, rs in g.items() if max(rs[0]["d1"], rs[0]["d2"]) >= minmax]
    base = {}
    for blk, rs in blks:
        m, n = rs[0]["d1"], rs[0]["d2"]
        D = np.array([P.D_units(r) for r in rs])
        F0 = np.array([[s2_reg.feats(r["x"], m, n)[0][k] for k in ("rx", "eLx", "eL")] for r in rs])
        T = [terms(r["x"], m, n, hmode) for r in rs]
        base[blk] = (D, F0, T)
    res_all = {}
    for order in itertools.permutations("KPHX"):
        sds, cfs = [], []
        for blk, (D, F0, T) in base.items():
            R = np.array([rhos(t, order) for t in T])
            X = np.column_stack([F0, R])
            ok = [j for j in range(X.shape[1]) if X[:, j].std() > 0.3]
            A = np.column_stack([X[:, ok], np.ones(len(D))])
            cf, *_ = np.linalg.lstsq(A, D, rcond=None)
            sds.append((D - A @ cf).std())
            full = np.full(X.shape[1], np.nan)
            full[ok] = cf[:-1]
            cfs.append(full)
        cfs = np.array(cfs)
        res_all["".join(order)] = (np.median(sds), np.nanmedian(cfs, axis=0), np.mean(sds))
    for k, (md, cf, mn) in sorted(res_all.items(), key=lambda kv: kv[1][0]):
        print(f"{k} med res sd {md:6.2f} mean {mn:6.2f} | rx {cf[0]:+.2f} eLx {cf[1]:+.1f} eL {cf[2]:+.1f} rho {cf[3]:+.2f} {cf[4]:+.2f} {cf[5]:+.2f}")


if __name__ == "__main__":
    main()
