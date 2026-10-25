"""verify-r5-f-combine: an independent re-implementation of the frozen F-density candidates from their written
descriptions, used to check the worker's importable functions before scoring them.

  side/r: mx = x87(m*x); side 1 (mx <= n) r = x87(mx/n), side 2 r = x87(n/mx); s = x87(1 + r) (r > 0.375 only)
  P = sg * fl(c * LN x), c = a | b, sg = +1 | -1; H = -fl64(h * LN s)
  K_sd = exact K_side + delta/2^53, delta = r4-f-small-df sdmodel.delta (the frozen constant table, imported)
  M-sd   E = fl64(fl64(K_sd64 + P) + H), F = x87(fl(EXP(E)) / x)   (EXP's result stored as a double)
  M-sdm  F = x87(fl(EXP(E)) * fl(1/x));  M-sd1 / M-sdm1: E = fl64(K_sd + P + H) rounded once
  hkp-sd S1 = x87(fl(K_sd) + H), S2 = x87(S1 + P), E = fl64(S2 + (K_sd - fl(K_sd))), F as M-sd
Excel's LN comes from ../excelmath (the engine harness), EXP from r3-loader-bd0 r3lib.exp_v6 (extended result).
"""

from __future__ import annotations

import pickle
import sys
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T, T / "r3-loader-bd0", T / "r4-f-small-df"):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import excelmath as XM  # noqa: E402

mpf = mpmath.mpf
_CF = HERE / "ln_cache.pkl"
_LN: dict = pickle.loads(_CF.read_bytes()) if _CF.exists() else {}


def prefetch_ln(vs) -> None:
    need = sorted({float(v) for v in vs} - set(_LN))
    for i in range(0, len(need), 4000):
        chunk = need[i : i + 4000]
        _LN.update(zip(chunk, XM.excel("LN", chunk)))
    if need:
        _CF.write_bytes(pickle.dumps(_LN))


def LN(v: float) -> float:
    if v not in _LN:
        prefetch_ln([v])
    return _LN[v]


def side_r(x: float, m: float, n: float):
    mx = XM.x87("*", m, x)
    return (1, XM.x87("/", mx, n)) if mx <= n else (2, XM.x87("/", n, mx))


def ln_inputs(x: float, m: float, n: float):
    _, r = side_r(x, m, n)
    return [x, XM.x87("+", 1.0, r)]


_K: dict = {}


def K_sd(m: float, n: float, side: int):
    key = (m, n, side)
    if key not in _K:
        import sdmodel  # the frozen constant table only

        a, b = mpf(m) / 2, mpf(n) / 2
        lnB = mpmath.log(mpmath.beta(a, b))
        t = mpmath.log(mpf(m) / mpf(n))
        K = -lnB + (a * t if side == 1 else -b * t)
        _K[key] = K + mpf(sdmodel.delta(m, n, side)) / mpf(2) ** 53
    return _K[key]


_KS: dict = {}


def K_seq(m: float, n: float, side: int):
    """K_sd as an x87 register holds it: fl64(fl64(exact K) + fl64(delta * 2^-53))."""
    key = (m, n, side)
    if key not in _KS:
        import sdmodel

        d = XM.fl64(mpf(sdmodel.delta(m, n, side)) / mpf(2) ** 53)
        _KS[key] = XM.fl64(XM.fl64(K_sd(m, n, side) - mpf(sdmodel.delta(m, n, side)) / mpf(2) ** 53) + d)
    return _KS[key]


def _exp64(E):
    """EXP V6 on the extended E (an fl64 mpf, exactly representable as a longdouble); the exact extended result."""
    import r3lib

    p, q = r3lib.exp_v6(r3lib._ld_of(E), True).as_integer_ratio()
    return mpf(p) / q


def value(x: float, m: float, n: float, model: str) -> float:
    """model: M-sd, M-sdm, M-sd1, M-sdm1, hkp-sd, optionally suffixed '+xexp' (EXP's result kept extended) or
    '+dexp' (stored as a double); the default is each frozen model's own (M-*: double, hkp-sd: extended)."""
    model, _, ex = model.partition("+")
    dexp = model.startswith("M-sd") if not ex else ex == "dexp"
    side, r = side_r(x, m, n)
    if r <= 0.375:
        raise ValueError("series branch not re-implemented")
    a, b, h = m / 2, n / 2, (m + n) / 2
    s = XM.x87("+", 1.0, r)
    P = (1 if side == 1 else -1) * XM.fl53(mpf(a if side == 1 else b) * mpf(LN(x)))
    H = -XM.fl64(mpf(h) * mpf(LN(s)))
    K = K_sd(m, n, side)
    if model in ("M-sd", "M-sdm"):  # x87 order: K held as fl64(fl64(K') + fl64(delta)), then + P, then + H
        K64 = K_seq(m, n, side)
        E = XM.fl64(XM.fl64(K64 + mpf(P)) + H)
    elif model in ("M-sd1", "M-sdm1"):  # the same sum rounded once (exact K_sd)
        E = XM.fl64(K + mpf(P) + H)
    elif model == "hkp-sd":
        khi = XM.fl53(K)
        S1 = XM.fl53(XM.fl64(mpf(khi) + H))
        S2 = XM.fl53(XM.fl64(mpf(S1) + mpf(P)))
        E = XM.fl64(mpf(S2) + (K - mpf(khi)))
    else:
        raise ValueError(model)
    w = _exp64(E)
    if dexp:  # M-sd stores EXP's result as a double before the 1/x step (sdmodel / slib.EXP)
        w = mpf(XM.fl53(w))
    if model in ("M-sdm", "M-sdm1"):
        return XM.fl53(XM.fl64(w * mpf(XM.x87("/", 1.0, x))))
    return XM.fl53(XM.fl64(w / mpf(x)))
