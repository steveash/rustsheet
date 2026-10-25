"""T-2586 r5-f-edges: reading the hidden addend Q of Excel's stored sum fl(Q + P) from the tiny-x / huge-x cells.

Where H = -h L(r) is negligible (r < 1E-18 / h) the exponent is E = fl(Q + P) + R, P = fl(c LN x) a double on the
grid q = ulp(P). The needed constant D = E_needed - P - H - K' (units 2^-53) is then C - psi_q(Q), psi_q(Q) = Q's
offset from its nearest multiple of q, so the D values at different q read Q's low bits. `groups(name)` gives the
per-(shape, side) cells, `pred(Q, q)` the offset, `spread(cells, Q)` the residual range after one free C."""

from __future__ import annotations

import math

import mpmath

import edgeprobe as MK
import elib as E
import s4_probe as S4

L = E.L
mpf = L.mpf
LDB, st = L.LDB, L.st
U = mpf(2) ** -53


def ex(v) -> mpmath.mpf:
    p, q = LDB(v).as_integer_ratio()
    return mpf(p) / q


def ulp_units(v: float) -> mpmath.mpf:
    """ulp of the double v, in units of 2^-53 (absolute)."""
    m, e = math.frexp(abs(v))
    return mpf(2) ** (e - 53) / U


def P_of(x: float, m: float, n: float, side: int, form: str = "x") -> float:
    """The stored power term: 'x' fl(c LN x) (M-sd), 'mx' fl(c LN(fl(m x))), c = a | -b."""
    c = m / 2 if side == 1 else -n / 2
    if form == "x":
        return st(LDB(c) * L.LN(x))
    if form == "x1":  # fl((c - 1) LN x), the 1/x inside the exponent (r3's form)
        return st(LDB(c - 1) * L.LN(x))
    return st(LDB(c) * L.LN(st(LDB(m) * LDB(x))))


def cell(o, form: str = "x"):
    """dict(x, m, n, side, q, D, w) for a probe row in the clean region, else None. D = E_needed - P - H - K' (form
    'x'); for form 'mx' the same with that P (D then also carries the constant c ln m, absorbed by C)."""
    e, x = S4.excel_of(o), S4.x_of(o)
    if not isinstance(e, float) or e == 0 or x <= 0:
        return None
    m, n = float(math.trunc(o["d1"])), float(math.trunc(o["d2"]))
    side, r = L.side_r(x, m, n)
    h = (m + n) / 2
    if r * h > 1e-18:
        return None
    A, Kex, _ = L.chain(x, m, n, "x" if form == "x1" else "xd")
    iv = E.e_needed(x, e, "exp" if form == "x1" else "div")
    if iv is None:
        return None
    E0, lo, hi = iv
    c = m / 2 if side == 1 else -n / 2
    P = P_of(x, m, n, side, form)
    D = (ex(E0) - ex(A) - Kex) / U + mpf(lo + hi) / 2
    if form == "mx":
        D += (ex(st(LDB(c) * L.LN(x))) - ex(P)) / U
    return dict(x=x, m=m, n=n, side=side, q=ulp_units(P), D=D, w=(hi - lo) / 2, row=o["row"])


def groups(name: str = "a", kinds=("tiny", "thresh", "huge", "hthr", "sub"), form: str = "x") -> dict:
    out: dict = {}
    for o in MK.rows(name):
        if o["kind"] in kinds:
            c = cell(o, form)
            if c:
                out.setdefault((c["m"], c["n"], c["side"]), []).append(c)
    return out


def pred(Q, q) -> mpmath.mpf:
    """round(Q/q)*q - Q in units, Q in absolute (mpf), q in units."""
    qu = Q / U / q
    return (mpmath.nint(qu) - qu) * q


def spread(cells, Q) -> float:
    res = [c["D"] - pred(Q, c["q"]) for c in cells]
    return float(max(res) - min(res))
