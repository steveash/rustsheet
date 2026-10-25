"""Loader (2000) saddle-point pieces with pluggable arithmetic, for the a, b > 1 density.

dbinom_raw(k, n, p, q) = exp(lc - lf/2), lc = stirlerr(n) - stirlerr(k) - stirlerr(n-k) - bd0(k, n p) - bd0(n-k, n q).
The beta density is (a+b-1) * dbinom_raw(a-1, a+b-2, x, y).  Here the x-dependent part X = -bd0(k, n p) - bd0(n-k, n q)
is modelled exactly per arithmetic; the shape-only part is either Loader's (stirlerr table from w6's h2_loader) or a
free constant (scan).

Arithmetic `ar` is one of dlib.rnd's modes: 'x' (x87 stored), 'e' (extended), 'd' (SSE double), 'm' (exact).
"""

from __future__ import annotations

import mpmath

import dlib as L

mpf = mpmath.mpf


def R(v, ar):
    return L.rnd(v, ar)


def ln(v, lnmode, ar):
    """log for bd0: 'xl' Excel LN of the stored double argument; 'e' FYL2X kept extended; 'm' exact."""
    v = mpf(v)
    if lnmode == "xl":
        return mpf(L.LN(float(L.R53(v))))
    if lnmode == "e":
        return L.ln64(v)
    return mpmath.log(v)


def bd0(x, np_, ar="x", lnmode="xl", cut=0.1, series="R", maxit=1000, addar=None, sar=None):
    """R's bd0: Taylor series in v = (x-np)/(x+np) when |x-np| < cut*(x+np), else x ln(x/np) + np - x.
    series 'R': s = d*v; ej = 2 x v; v^2; s += ej/(2j+1) (R).  series 'R2': s starts at d*v, ej updated first."""
    x, np_ = mpf(x), mpf(np_)
    d = R(x - np_, ar)
    sm = R(x + np_, ar)
    if abs(d) < R(cut * sm, ar):
        if series == "lp":  # x ln(x/np) with ln = Excel's log1p (atanh series) of a = d/np, then + (np - x)
            import excel_log1p

            aa = float(L.D(d / np_))
            t = R(x * mpf(excel_log1p.log1p_series(aa)), ar)
            return R(t - d, addar or ar)
        if series == "lpv":  # same series entered with its variable v = d/(x+np) directly
            import excel_log1p

            r = L.D(d / sm)
            l = 2 * L.D(mpf(r) + mpf(_atanh_tail(r)))
            t = R(x * l, ar)
            return R(t - d, addar or ar)
        if sar is not None:  # series in its own arithmetic
            ar = sar
        v = R(d / sm, ar)
        s = R(d * v, ar)
        ej = R(R(2 * x, ar) * v, ar)
        v2 = R(v * v, ar)
        for j in range(1, maxit):
            ej = R(ej * v2, ar)
            s1 = R(s + R(ej / (2 * j + 1), ar), ar)
            if s1 == s:
                return s1
            s = s1
        return s
    q = R(x / np_, ar)
    t = R(x * ln(q, lnmode, ar), ar)
    if addar is None:
        return R(R(t + np_, ar) - x, ar)
    return R(t + R(np_ - x, addar), addar)  # t + (np - x), e.g. in extended


def _atanh_tail(r: float, N: int = 14) -> float:
    """r2-log1p's series tail s = sum c_k r^(2k+1), stored x87 ops, for an arbitrary stored variable r."""
    from excel_log1p import CS
    from x87f import x87 as X

    t2 = X("*", r, r)
    p, s = r, None
    for k in range(1, N + 1):
        p = X("*", p, t2)
        tm = X("*", p, CS[k])
        s = tm if s is None else X("+", s, tm)
    return s


def xpart(x, a, b, ar="x", lnmode="xl", ymode="fl", npmode="nx", order="pq", xar=None, **kw):
    """X = -bd0(k, n p) - bd0(n-k, n q) with k = a-1, n = a+b-2, p = x, q = fl(1-x) (ymode 'fl') or exact."""
    X = mpf(x)
    k = mpf(L.D(mpf(a) - 1))
    n = mpf(L.D(mpf(L.D(mpf(a) + mpf(b))) - 2))
    nk = mpf(L.D(n - k))
    q = mpf(1.0 - x) if ymode == "fl" else 1 - X
    if npmode == "nx":
        npx, nqx = R(n * X, ar), R(n * q, ar)
    elif npmode == "sx2":  # n x formed as (a+b) x - 2x, n q as n fl(1-x)
        sab = mpf(L.D(mpf(a) + mpf(b)))
        npx, nqx = R(R(sab * X, ar) - 2 * X, ar), R(n * q, ar)
    elif npmode == "sx2q":  # both from (a+b): n x = (a+b)x - 2x, n q = (a+b) q - 2q
        sab = mpf(L.D(mpf(a) + mpf(b)))
        npx, nqx = R(R(sab * X, ar) - 2 * X, ar), R(R(sab * q, ar) - 2 * q, ar)
    else:  # n q formed as n - n p
        npx = R(n * X, ar)
        nqx = R(n - npx, ar)
    b1 = bd0(k, npx, ar, lnmode, **kw)
    b2 = bd0(nk, nqx, ar, lnmode, **kw)
    xa = xar or ar
    if order == "pq":
        return R(-b1 - b2, xa), b1, b2
    return R(-b2 - b1, xa), b1, b2
