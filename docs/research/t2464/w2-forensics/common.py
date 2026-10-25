"""T-2464 w2-forensics shared helpers: data loading, exact values, Excel elementary functions, scoring."""
from __future__ import annotations

import csv
import math
import sys
from functools import lru_cache
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
sys.path.insert(0, str(T2464))
import excelmath as em  # noqa: E402

mpmath.mp.prec = 300
U = 2.0 ** -53


def load_beta_exact():
    rows = []
    for r in csv.DictReader(open(T2464 / "beta_exact.tsv"), delimiter="\t"):
        rows.append(dict(case=r["case"], cell=r["cell"], x=float(r["x"]), a=float(r["a"]), b=float(r["b"]),
                         cum=r["cum"] == "1", excel=float(r["excel"]), exact=float(r["exact"]), branch=r["branch"]))
    return rows


def rel_err_units(got: float, exact_mp) -> float:
    """(got - exact)/exact in units of 2^-53."""
    return float((mpmath.mpf(got) - exact_mp) / exact_mp) / U


@lru_cache(maxsize=None)
def exact_cdf(x: float, a: float, b: float):
    return mpmath.betainc(mpmath.mpf(a), mpmath.mpf(b), 0, mpmath.mpf(x), regularized=True)


@lru_cache(maxsize=None)
def exact_pdf(x: float, a: float, b: float):
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    return X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B)


@lru_cache(maxsize=None)
def exact_lnbeta(a: float, b: float):
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    return mpmath.loggamma(A) + mpmath.loggamma(B) - mpmath.loggamma(A + B)


_cache: dict[tuple[str, float], float] = {}


def xl(name: str, xs) -> list[float]:
    """Excel's NAME(x) for each x, memoized, batched."""
    need = sorted({float(x) for x in xs if (name, float(x)) not in _cache})
    for i in range(0, len(need), 200000):
        chunk = need[i:i + 200000]
        for x, v in zip(chunk, em.excel(name, chunk)):
            _cache[(name, x)] = v
    return [_cache[(name, float(x))] for x in xs]


fl64, fl53 = em.fl64, em.fl53


def x87(op: str, a, b) -> float:
    A, B = mpmath.mpf(a), mpmath.mpf(b)
    if op == "+":
        r = A + B
    elif op == "-":
        r = A - B
    elif op == "*":
        r = A * B
    else:
        r = A / B
    return fl53(fl64(r))


def within15(got: float, excel: float) -> bool:
    if got == excel:
        return True
    if excel == 0 or not math.isfinite(got):
        return False
    e = math.floor(math.log10(abs(excel)))
    return abs(got - excel) <= 0.5 * 10 ** (e - 14)


def ulpdiff(got: float, excel: float) -> float:
    if got == excel:
        return 0
    return abs(got - excel) / math.ulp(excel)


def score(pairs) -> tuple[int, int, int, int]:
    """pairs of (model, excel): (n, exact, within 4 ulp, within 15 digits)."""
    n = ex = w4 = w15 = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        w4 += ulpdiff(m, e) <= 4
        w15 += within15(m, e)
    return n, ex, w4, w15


def ols(y, cols: dict[str, np.ndarray]):
    names = list(cols)
    X = np.column_stack([cols[k] for k in names] + [np.ones(len(y))])
    coef, *_ = np.linalg.lstsq(X, y, rcond=None)
    res = y - X @ coef
    # standard errors
    dof = max(1, len(y) - X.shape[1])
    s2 = res @ res / dof
    try:
        cov = s2 * np.linalg.inv(X.T @ X)
        se = np.sqrt(np.diag(cov))
    except np.linalg.LinAlgError:
        se = np.full(len(coef), np.nan)
    return {n: (c, s) for n, c, s in zip(names + ["const"], coef, se)}, float(np.std(y)), float(np.std(res))
