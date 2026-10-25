"""T-2586 r4-f-small-df: the frozen F density model M-sd (frozen 2026-10-24 before the hold-out was read).

M-sd = r4-f-brcomp's M-xd x-structure with a measured constant:
  E = K + fl(c LN x) - h L(r) (extended), F = fl(EXP(E) / x),   c = a (side 1, m x <= n) | -b (side 2),
  K = K_exact + delta(m, n, side) units, where
  delta = e(h) - e(a) - e(b) + C0 + [(h - 1/2) L(fl(w/o)) - (h - 1/2) ln(h/o)],   (w, o) = (a, b) side 1 | (b, a) side 2
  - e(v): r3-loader-const's per-half-integer stirlerr error table (grid_e.tsv, v in 1.5..28), e(0.5) = E05,
    e(1) = E1 (fitted), linear gauge extrapolation above 28;
  - the bracket: the rounding of Excel's (h - 1/2) ln(h/o) when it is formed as (h - 1/2) * log1p(fl(w/o)) with
    Excel's log1p (series to 0.375, LN(fl(1 + w/o)) above), product kept extended.
  Fitted on probe t2586-r4-f-small-df-a and r3-tdensity's d1 = 1 runs (s19_sidefit.py: RMS 0.52 units, 55/56 groups
  within 1 unit)."""

from __future__ import annotations

import functools

import numpy as np

import slib as L

LDB, st, mp = L.LDB, L.st, L.mpmath
C0, E05, E1 = 5.33, 0.22, 0.06
_E = {float(v): float(e) for v, e in (ln.split("\t") for ln in (L.T / "r3-loader-const" / "grid_e.tsv").read_text().splitlines()[1:])}
_hi = sorted(v for v in _E if v >= 14)
_ALPHA, _GAMMA = np.polyfit(_hi, [_E[v] for v in _hi], 1)


def e_of(v: float) -> float:
    if v == 0.5:
        return E05
    if v == 1.0:
        return E1
    if v in _E:
        return _E[v]
    return float(_ALPHA * v + _GAMMA)


@functools.lru_cache(maxsize=None)
def delta(m: float, n: float, side: int) -> float:
    a, b, h = m / 2, n / 2, (m + n) / 2
    w, o = (a, b) if side == 1 else (b, a)
    lp = L.L_xl(st(LDB(w) / LDB(o)))
    p, q = (LDB(h - 0.5) * lp).as_integer_ratio()
    side_err = float((L.mpf(p) / q - (h - 0.5) * mp.log(L.mpf(h) / o)) / L.U)
    return e_of(h) - e_of(a) - e_of(b) + C0 + side_err


@functools.lru_cache(maxsize=None)
def K_of(m: float, n: float, side: int) -> LDB:
    return L.ld(L.K_exact(m, n, side)) + LDB(delta(m, n, side)) * LDB(2.0) ** -53


def value(x: float, m: float, n: float, fin: str = "div") -> float:
    """fin 'div': fl(EXP(E)/x) (M-sd); 'mulr': fl(EXP(E) * fl(1/x)) (M-sd-mulr, the in-sample best bit-exact)."""
    side, r = L.side_r(x, m, n)
    if fin == "div":
        return L.mxd_K(x, m, n, K_of(m, n, side))
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if side == 1 else -b
    E = K_of(m, n, side) + LDB(st(LDB(c) * L.LN(x))) - LDB(h) * L.L_xl(r)
    return st(LDB(L.EXP(E)) * LDB(st(LDB(1) / LDB(x))))
