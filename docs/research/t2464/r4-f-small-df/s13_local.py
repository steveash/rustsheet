"""Local-rounding regression at small shapes: target = centre of Excel's admissible E interval under F = EXP(E)
(units, absolute, relative to the exact ln F); regressors = rounding errors (absolute units) of candidate Excel
pieces. Per run least squares with intercept; prints coefficients and the residual sd.
Usage: s13_local.py maxd [reg ...]"""
import sys

import numpy as np

import slib as L

LDB, st, LN = L.LDB, L.st, L.LN
U = L.U
maxd = float(sys.argv[1])
REGS = sys.argv[2:] or ["LNx", "LNmx", "LNnmx", "LNr", "L"]
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]
grid = np.arange(-24, 24.0001, 0.02).astype(LDB)
UL = LDB(2.0) ** -53


def fr(v):
    p, q = LDB(v).as_integer_ratio()
    return L.mpf(p) / q


def errs(x, m, n):
    X, M, N = L.mpf(x), L.mpf(m), L.mpf(n)
    s, r = L.side_r(x, m, n)
    mx = st(LDB(m) * LDB(x))
    rex = M * X / N if s == 1 else N / (M * X)
    e = {}
    e["LNx"] = fr(LN(x)) - L.mpmath.log(X)
    e["LNmx"] = fr(LN(mx)) - L.mpmath.log(M * X)
    e["LNnmx"] = fr(LN(st(LDB(n) + LDB(mx)))) - L.mpmath.log(N + M * X)
    e["LNr"] = fr(LN(r)) - L.mpmath.log(rex)
    e["L"] = fr(L.L_xl(r)) - L.mpmath.log1p(rex)
    e["LN1r"] = fr(LN(st(LDB(1) + LDB(r)))) - L.mpmath.log1p(rex)
    y = st(LDB(mx) / LDB(st(LDB(mx) + LDB(n))))
    q = st(LDB(n) / LDB(st(LDB(mx) + LDB(n))))
    e["LNy"] = fr(LN(y)) - L.mpmath.log(M * X / (M * X + N))
    e["LNq"] = fr(LN(q)) - L.mpmath.log(N / (M * X + N))
    a, b = m / 2, n / 2
    for nm, c in (("Pst", a if s == 1 else -b), ("Pst1", (a - 1) if s == 1 else -(b + 1)), ("Pr", a if s == 1 else b)):
        lnv = LN(x) if nm != "Pr" else LN(r)
        e[nm] = L.mpf(st(LDB(c) * lnv)) - fr(LDB(c) * lnv)
    return s, {k: float(v / U) for k, v in e.items()}


g: dict = {}
for r in rows:
    x = r["x"]
    Es = L.mpmath.log(r["exact"])
    v = L.exp_v6_vec(L.ld(Es) + grid * UL)
    ok = grid[v == r["excel"]]
    if not len(ok):
        continue
    s, e = errs(x, r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s, r["blk"]), []).append([0.5 * float(ok.min() + ok.max())] + [e[k] for k in REGS])
print("run | sd | " + " ".join(f"{k:>7s}" for k in REGS) + " | res sd")
for (m, n, s, b), v in sorted(g.items()):
    a = np.array(v)
    X = np.column_stack([np.ones(len(a))] + [a[:, i + 1] for i in range(len(REGS))])
    coef, *_ = np.linalg.lstsq(X, a[:, 0], rcond=None)
    res = a[:, 0] - X @ coef
    h = (m + n) / 2
    print(f"({m:g},{n:g}) s{s} {b[-6:]:6s} a {m/2:.1f} b {n/2:.1f} h {h:.1f} | {a[:, 0].std():5.2f} | "
          + " ".join(f"{c:+7.2f}" for c in coef[1:]) + f" | {res.std():5.2f}")
