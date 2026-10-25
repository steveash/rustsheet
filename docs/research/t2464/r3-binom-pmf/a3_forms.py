"""Chain-free form test on interior BINOM pmf cells: rms of D - dX, where D is Excel's exponent error and dX the
candidate's (both in 2^-53 units, absolute in E).  A form that is Excel's leaves only the output rounding (~0.3-0.6)."""
import math
import sys

import numpy as np

import blib as B
import r3lib as R3

LDB = np.longdouble
st = B.st
rows = [r for r in B.load_cells() if r["fn"] == "BIN" and 0 < r["k"] < r["n"] and 1e-5 < r["p"] and r["n"] <= 1000]
gl = B.GL([v for r in rows for v in (r["n"] + 1.0, r["k"] + 1.0, r["n"] - r["k"] + 1.0)])
G = dict(zip([v for r in rows for v in (r["n"] + 1.0, r["k"] + 1.0, r["n"] - r["k"] + 1.0)], gl))


def direct(r, o):
    k, n, p = r["k"], r["n"], r["p"]
    q = st(LDB(1) - LDB(p))
    lp = LDB(B.LN(p))
    lq = LDB(B.LN(q)) if o.get("lq") == "LN" else LDB(B.log1p_xl(-p))
    X = lambda v, m: LDB(st(v)) if m == "x" else v  # noqa: E731
    m = o.get("m", "x")
    if o.get("C") == "GL":
        C = X(X(LDB(G[n + 1.0]) - LDB(G[k + 1.0]), m) - LDB(G[n - k + 1.0]), m)
    else:
        C = LDB(float(B.mpmath.log(math.comb(n, k))))
    t1 = X(LDB(k) * lp, m)
    t2 = X(LDB(n - k) * lq, m)
    return X(X(C + t1, m) + t2, m) if o.get("ord", "Ct") == "Ct" else X(C + X(t1 + t2, m), m)


forms = {
    "loader r3 (stir ex)": lambda r: B.loader_E(r["k"], r["n"], r["p"], st(LDB(1) - LDB(r["p"])), dict(stir="ex", chain="e")),
    "loader r3 (stir R)": lambda r: B.loader_E(r["k"], r["n"], r["p"], st(LDB(1) - LDB(r["p"])), dict(stir="R", chain="e")),
    "loader r3 stir R, lf exact": lambda r: B.loader_E(r["k"], r["n"], r["p"], st(LDB(1) - LDB(r["p"])), dict(stir="R", chain="e", lf="ex")),
    "direct GL, LN q, x": lambda r: direct(r, dict(C="GL", lq="LN")),
    "direct GL, log1p q, x": lambda r: direct(r, dict(C="GL")),
    "direct GL, log1p q, e": lambda r: direct(r, dict(C="GL", m="e")),
    "direct exactC, log1p q, x": lambda r: direct(r, dict()),
    "direct exactC, LN q, x": lambda r: direct(r, dict(lq="LN")),
    "direct GL, log1p, C+(t1+t2)": lambda r: direct(r, dict(C="GL", ord="t")),
}
Ds = [B.D_units(r) for r in rows]
print("cells", len(rows), "rms D (CR baseline)", math.sqrt(sum(d * d for d in Ds) / len(Ds)))
for name, f in forms.items():
    res = []
    for r, d in zip(rows, Ds):
        res.append(d - B.units(f(r), r))
    rms = math.sqrt(sum(x * x for x in res) / len(res))
    med = sorted(abs(x) for x in res)[len(res) // 2]
    big = sum(abs(x) > 3 for x in res)
    print(f"{name:32s} rms {rms:8.2f}  median|.| {med:6.2f}  >3 units {big}")
