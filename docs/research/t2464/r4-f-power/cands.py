"""Candidate stored quantities of Excel's F density: each returns (computed value with Excel's stored x87 ops,
exact value, sign as it enters lnF).  Errors are absolute in log space, in 2^-53 units."""
from __future__ import annotations

import mpmath

import plib as P

F = P.F
LDB, st = F.LDB, F.st
mpf = mpmath.mpf
U = P.U


def S(v):
    return LDB(st(v))


def ex(v):
    if isinstance(v, mpmath.mpf):
        return v
    p, q = LDB(v).as_integer_ratio()
    return mpf(p) / q


def LNx(v):
    return F.LN(v)


def L1P(r):
    """Excel's log1p (series to 0.375, LN(fl(1+r)) above), as a longdouble."""
    return LDB(F.TD.log1p_xl(float(r)))


def pieces(x, m, n):
    """Excel-stored intermediate pieces and exact references."""
    X, M, N = mpf(x), mpf(m), mpf(n)
    a, b, h = m / 2, n / 2, (m + n) / 2
    mx = st(LDB(m) * LDB(x))
    r2 = st(LDB(mx) / LDB(n))  # mx/n
    r1 = st(LDB(n) / LDB(mx))  # n/mx
    den = st(LDB(n) + LDB(mx))
    e = dict(lx=mpmath.log(X), lr2=mpmath.log(M * X / N), l1p2=mpmath.log1p(M * X / N), l1p1=mpmath.log1p(N / (M * X)))
    s = dict(mx=mx, r1=r1, r2=r2, den=den, a=a, b=b, h=h)
    s["Lx"] = LNx(x)
    s["Lr2"] = LNx(r2)
    s["Lr1"] = LNx(r1)
    s["L1p2"] = L1P(r2)
    s["L1p1"] = L1P(r1)
    s["Lmx"] = LNx(mx)
    s["Lden"] = LNx(den)
    s["Ln"] = LNx(n)
    s["Lm"] = LNx(m)
    return s, e


def cand_terms(x, m, n):
    """{name: (computed, exact)} for log-space terms, signed as they enter lnF."""
    s, e = pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    A, B, H = mpf(a), mpf(b), mpf(h)
    out = {}

    def prod(c, L):
        return S(LDB(c) * L)

    lx = e["lx"]
    for nm, c in (("a-1", a - 1), ("a", a), ("b", b), ("b+1", b + 1), ("h", h)):
        if c != 0:
            out[f"{nm}*LN(x)"] = (prod(c, s["Lx"]), mpf(c) * lx)
    out["a*LN(r2)"] = (prod(a, s["Lr2"]), A * e["lr2"])
    out["b*LN(r2)"] = (prod(b, s["Lr2"]), B * e["lr2"])
    out["h*L1P(r2)"] = (prod(h, s["L1p2"]), H * e["l1p2"])
    out["h*L1P(r1)"] = (prod(h, s["L1p1"]), H * e["l1p1"])
    out["a*L1P(r1)"] = (prod(a, s["L1p1"]), A * e["l1p1"])
    out["b*L1P(r2)"] = (prod(b, s["L1p2"]), B * e["l1p2"])
    # sums a ln y + b ln q in the ratio forms
    aly = prod(a, s["L1p1"])  # -a ln y
    blq = prod(b, s["L1p2"])  # -b ln q
    out["aLy+bLq"] = (S(aly + blq), A * e["l1p1"] + B * e["l1p2"])
    # b ln(r2) + h L1P(r1)  (= -(a ln y + b ln q))
    out["bLr2+hL1p1"] = (S(prod(b, s["Lr2"]) + prod(h, s["L1p1"])), B * e["lr2"] + H * e["l1p1"])
    out["aLr1... a*LN(r1)+h*L1P(r2)"] = (S(prod(a, s["Lr1"]) + prod(h, s["L1p2"])), -A * e["lr2"] + H * e["l1p2"])
    out["aLr2-hL1p2"] = (S(prod(a, s["Lr2"]) - prod(h, s["L1p2"])), A * e["lr2"] - H * e["l1p2"])
    out["(a-1)Lx-hL1p2"] = (S(prod(a - 1, s["Lx"]) - prod(h, s["L1p2"])), (A - 1) * lx - H * e["l1p2"])
    out["(b+1)Lx+hL1p1"] = (S(prod(b + 1, s["Lx"]) + prod(h, s["L1p1"])), (B + 1) * lx + H * e["l1p1"])
    return out


def errs(x, m, n):
    """{name: (stored - exact) in units}."""
    return {k: float((ex(c) - v) * U) for k, (c, v) in cand_terms(x, m, n).items()}
