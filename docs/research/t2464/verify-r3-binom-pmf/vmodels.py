"""Candidate values for the verifier.

Own implementations (from excelmath's bit-exact EXP/LN and x87 rounding, plus r2-log1p's verified series):
- edge_kn(n, p) = EXP(fl(n * LN(p)))                 (claim: BINOM.DIST(n,n,p,FALSE))
- edge_k0(n, p) = EXP(fl(n * ln1m(p)))               (claim: BINOM.DIST(0,n,p,FALSE); ln1m = r2-log1p's)
- R's dbinom_raw edge branches as the rival: k = n with q < 0.1: exp(-bd0(n, np) - nq);
  k = 0 with p < 0.1: exp(-bd0(n, nq) - np), with bd0 exact and the exponent rounded once (the most favourable
  version of the rival).
Interior models: r3-binom-pmf's frozen `blib.model_value` (read and checked against its description first).
"""

from __future__ import annotations

import math
import sys
from fractions import Fraction
from pathlib import Path

import mpmath

T = Path(__file__).resolve().parent.parent
for d in (T, T / "r2-log1p"):
    if str(d) not in sys.path:
        sys.path.insert(0, str(d))

import excelmath as XM  # noqa: E402
import excel_log1p as L1P  # noqa: E402

_memo: dict = {}


def xl(name: str, xs) -> list[float]:
    """Excel's NAME(x), batched and memoised."""
    need = sorted({float(x) for x in xs if (name, float(x)) not in _memo})
    for i in range(0, len(need), 4000):
        chunk = need[i : i + 4000]
        for x, v in zip(chunk, XM.excel(name, chunk)):
            _memo[(name, x)] = v
    return [_memo[(name, float(x))] for x in xs]


def ln1m(p: float) -> float:
    """Excel's ln(1-p): r2-log1p's series for p <= 0.375, else LN(fl(1-p)) (stored x87 subtraction)."""
    if p <= 0.375:
        return L1P.log1p_series(-p)
    return xl("LN", [XM.x87("-", 1.0, p)])[0]


def edge_exponent(k: int, n: int, p: float) -> float:
    """The stored exponent of the claimed closed forms."""
    if k == n:
        return XM.x87("*", float(n), xl("LN", [p])[0])
    return XM.x87("*", float(n), ln1m(p))


def edge_values(rows) -> list[float]:
    es = [edge_exponent(r["k"], r["n"], float(r["p"])) for r in rows]
    return xl("EXP", es)


def bd0_exact(x, M):
    return x * mpmath.log(x / M) + M - x


def r_edge(k: int, n: int, p: Fraction) -> float | None:
    """R's dbinom_raw edge branch value when it differs in form from the closed form, else None."""
    P = mpmath.mpf(p.numerator) / p.denominator
    Q = 1 - P
    if n == 0:
        return None
    if k == n and Q < 0.1:
        return float(mpmath.exp(-bd0_exact(mpmath.mpf(n), n * P) - n * Q))
    if k == 0 and P < 0.1:
        return float(mpmath.exp(-bd0_exact(mpmath.mpf(n), n * Q) - n * P))
    return None


_B = None


def blib():
    global _B
    if _B is None:
        sys.path.insert(0, str(T / "r3-binom-pmf"))
        import blib as B  # r3-binom-pmf's frozen models (candidate function only)

        _B = B
    return _B


def worker_model(name: str, r) -> float | None:
    """r3-binom-pmf's frozen model on one row, in the worker's convention: NEG rows go in as (k=r, n=f+r)."""
    B = blib()
    k, n = (r["n"], r["k"] + r["n"]) if r["kern"] == "NEG" else (r["k"], r["n"])
    try:
        v = B.model_value(name, r["kern"], k, n, float(r["p"]))
    except (ValueError, ZeroDivisionError, OverflowError):
        return None
    return v if math.isfinite(v) else None
