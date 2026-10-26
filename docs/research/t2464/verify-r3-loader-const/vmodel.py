"""verify-r3-loader-const: an independent implementation of the worker's M3 / M4 density, written from RESULTS.md's
description (not from m1_model.py), plus the reference variants it is compared with.

  k = fl(a-1), n = fl(fl(a+b)-2), nk = fl(n-k)                       (stored x87 ops: fl53(fl64(.)))
  M1 = fl(n x), M2 = fl(n fl(1-x))
  bd0(k, M): d = fl(k-M), s = fl(k+M); |d| < fl(0.2 s): series in extended (v = d/s, R's recursion, RN64 each op);
             else log form: t = fl(k * LN(fl(k/M))), bd0 = RN64(t + RN64(M - k))
  X  = RN64(-bd0_1 - bd0_2)        (M4: each bd0 rounded to a double first)
  C  = LN(fl(n+1)) + st(n) - st(k) - st(nk) - lf/2, st = exact stirlerr,
       lf = RN64(RN64(RN64(ln2pi + LN k) + LN nk) - LN n)   (Excel LN pieces summed in extended)
  E  = C + X (unrounded) into Excel's EXP model V6.
Variants: chain 'ext' (as described), 'e64' (E rounded to 64 bits first), 'st' (E stored to a double), 'Cd' / 'Xd' /
'CdXd' (the constant and / or X stored to a double, their sum unrounded), 'L5x' / 'hx' / 'lcx' (one term added
last in extended to a stored rest; the verifier's own structure tests);
lf 'e' (extended), 'x' (stored ops), 'm' (exact); l5 'xl' (Excel LN) or 'm' (exact); cs 'exact' only.
Primitives: Excel LN from the engine harness (excelmath.excel, batched, cached in ln_cache.pkl); EXP V6 for an
extended input from w1-density's lib (positive-controlled against the harness on doubles in vscore.py).
"""

from __future__ import annotations

import pickle
import sys
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
for p in (T, T / "w1-density"):
    if str(p) not in sys.path:
        sys.path.append(str(p))

import mpmath  # noqa: E402

import excelmath  # noqa: E402
import lib as W1  # noqa: E402  (EXP V6 model on mpf inputs)

mpmath.mp.prec = 200
mpf = mpmath.mpf
fl64 = excelmath.fl64
LN2PI = mpf(float("1.837877066409345483560659472811"))  # the double nearest ln(2 pi) (C M_LN_2PI)


def st(v):
    """x87 stored op."""
    return mpf(excelmath.fl53(excelmath.fl64(v)))


# ------------------------------------------------------------------ Excel LN, batched
_LNP = HERE / "ln_cache.pkl"
_LN: dict = pickle.loads(_LNP.read_bytes()) if _LNP.exists() else {}


def ln_prefetch(vals):
    need = sorted({float(v) for v in vals if float(v) not in _LN})
    for i in range(0, len(need), 20000):
        ch = need[i : i + 20000]
        for v, r in zip(ch, excelmath.excel("LN", ch)):
            _LN[v] = r
    if need:
        _LNP.write_bytes(pickle.dumps(_LN))


def LN(v) -> mpf:
    return mpf(_LN[float(v)])


# ------------------------------------------------------------------ pieces
def pieces(a: float, b: float):
    A, B = mpf(a), mpf(b)
    k = st(A - 1)
    n = st(st(A + B) - 2)
    return k, n, st(n - k)


def sides(x: float, a: float, b: float):
    k, n, nk = pieces(a, b)
    X = mpf(x)
    q = mpf(1.0 - x)
    return [(k, st(n * X)), (nk, st(n * q))]


def is_log(kv, M) -> bool:
    d = st(kv - M)
    s = st(kv + M)
    return not abs(d) < st(mpf(0.2) * s)


def ln_needs(x: float, a: float, b: float):
    k, n, nk = pieces(a, b)
    out = [k, n, nk, st(n + 1)]
    for kv, M in sides(x, a, b):
        if is_log(kv, M):
            out.append(st(kv / M))
    return out


def bd0(kv, M):
    d = st(kv - M)
    s = st(kv + M)
    if abs(d) < st(mpf(0.2) * s):
        v = fl64(d / s)
        sm = fl64(d * v)
        ej = fl64(fl64(2 * kv) * v)
        v2 = fl64(v * v)
        for j in range(1, 1000):
            ej = fl64(ej * v2)
            s1 = fl64(sm + fl64(ej / (2 * j + 1)))
            if s1 == sm:
                return s1
            sm = s1
        return sm
    t = st(kv * LN(st(kv / M)))
    return fl64(t + fl64(M - kv))


def stirlerr(v):
    v = mpf(v)
    return mpmath.loggamma(v + 1) - (v + mpf(0.5)) * mpmath.log(v) + v - mpmath.log(2 * mpmath.pi) / 2


_C: dict = {}


def const(a: float, b: float, l5: str = "xl", lf: str = "e"):
    key = (a, b, l5, lf)
    if key not in _C:
        k, n, nk = pieces(a, b)
        L5 = LN(st(n + 1)) if l5 == "xl" else mpmath.log(n + 1)
        Cs = stirlerr(n) - stirlerr(k) - stirlerr(nk)
        if lf == "m":
            LF = mpmath.log(2 * mpmath.pi) + mpmath.log(k) + mpmath.log(nk) - mpmath.log(n)
        else:
            r = fl64 if lf == "e" else st
            LF = r(r(r(LN2PI + LN(k)) + LN(nk)) - LN(n))
        _C[key] = (L5, Cs, LF / 2)
    return _C[key]


def density(
    x: float, a: float, b: float, bd0st: bool = False, chain: str = "ext", l5: str = "xl", lf: str = "e"
):
    b1, b2 = (bd0(kv, M) for kv, M in sides(x, a, b))
    if bd0st:
        b1, b2 = st(b1), st(b2)
    X = fl64(-b1 - b2)
    L5, Cs, h = const(a, b, l5, lf)
    if chain == "L5x":  # L5 added last, in extended, to a stored rest
        return W1.exp_v6(L5 + st(Cs - h + X))
    if chain == "hx":  # lf/2 subtracted last, in extended, from a stored rest
        return W1.exp_v6(st(L5 + Cs + X) - h)
    if chain == "lcx":  # r2's grouping L5 + fl(fl(Cs + X) - lf/2), last add unrounded
        return W1.exp_v6(L5 + st(st(Cs + X) - h))
    C = L5 + Cs - h
    if chain in ("Cd", "CdXd"):  # constant stored to a double, X added in extended
        C = st(C)
    if chain in ("Xd", "CdXd"):
        X = st(X)
    E = C + X
    if chain == "e64":
        E = fl64(E)
    elif chain == "st":
        E = st(E)
    return W1.exp_v6(E)


MODELS = {
    "M3": dict(bd0st=False),
    "M4": dict(bd0st=True),
    "M4-e64": dict(bd0st=True, chain="e64"),
    "M4-stE": dict(bd0st=True, chain="st"),
    "M4-lfx": dict(bd0st=True, lf="x"),
    "Mexact": dict(bd0st=False, l5="m", lf="m"),
    "M4-Cd": dict(bd0st=True, chain="Cd"),
    "M4-Xd": dict(bd0st=True, chain="Xd"),
    "M4-CdXd": dict(bd0st=True, chain="CdXd"),
    "M4-L5x": dict(bd0st=True, chain="L5x"),
    "M4-hx": dict(bd0st=True, chain="hx"),
    "M4-lcx": dict(bd0st=True, chain="lcx"),
}


def exact_pdf(x: float, a: float, b: float) -> float:
    X, A, B = mpf(x), mpf(a), mpf(b)
    return float(X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B))
