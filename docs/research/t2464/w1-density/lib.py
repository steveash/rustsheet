"""T-2464 w1-density: shared helpers for the BETA.DIST density study.

Loads the 2,035 pdf cells of ../beta_exact.tsv, Excel's elementary functions (via ../excelmath.py, cached),
exact x87 helpers on mpmath values, an mpmath re-implementation of Excel's EXP model V6 and LN (FLDLN2/FYL2X)
that also accepts an extended (64-bit) input, and the scorer (exact / within 4 ulp / within 15 digits).
"""

from __future__ import annotations

import csv
import math
import os
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent))
import mpmath  # noqa: E402

import excelmath  # noqa: E402

mpmath.mp.prec = 300
mpf = mpmath.mpf
R64 = excelmath.fl64


def R53(v) -> float:
    return excelmath.fl53(v)


def D(v) -> float:
    """Stored x87 op: fl53(fl64(v))."""
    return excelmath.fl53(excelmath.fl64(v))


# ---------------------------------------------------------------- data
def load_pdf():
    rows = []
    for r in csv.DictReader(open(HERE.parent / "beta_exact.tsv"), delimiter="\t"):
        if r["cum"] != "0":
            continue
        rows.append(
            dict(
                case=r["case"].split("/")[-1].replace(".case.json", ""),
                cell=r["cell"],
                x=float(r["x"]),
                a=float(r["a"]),
                b=float(r["b"]),
                excel=float(r["excel"]),
                exact=float(r["exact"]),
            )
        )
    return rows


# ---------------------------------------------------------------- Excel functions (cached)
CACHE = HERE / "xcache.pkl"
_cache: dict = {}
if CACHE.exists():
    _cache = pickle.loads(CACHE.read_bytes())


def xl(name: str, xs):
    """Excel's NAME at each double, cached on disk."""
    need = sorted({float(x) for x in xs if (name, float(x)) not in _cache and math.isfinite(x)})
    if need:
        for i in range(0, len(need), 20000):
            chunk = need[i : i + 20000]
            for x, v in zip(chunk, excelmath.excel(name, chunk)):
                _cache[(name, x)] = v
        CACHE.write_bytes(pickle.dumps(_cache))
    return [_cache.get((name, float(x)), math.nan) for x in xs]


# ---------------------------------------------------------------- EXP V6 / LN in mpmath (extended input ok)
LOG2E64 = R64(1 / mpmath.ln(2))
LN2_64 = mpf(0xB17217F7D1CF79AC) * mpf(2) ** -64


def exp_v6(a) -> float:
    """Excel's EXP model V6 for an mpf input a (a double or an extended value)."""
    a = mpf(a)
    t = R64(a * LOG2E64)
    n = mpmath.nint(t)  # ties to even
    f = t - n
    u = R64(mpf(2) ** abs(f) - 1)
    w = R64(1 + u)
    if f < 0:
        w = R64(1 / w)
    return R53(w * mpf(2) ** n)


def exp_v6_ext(a):
    """EXP V6 kept in extended (no final store): w*2^n exactly, a 64-bit value."""
    a = mpf(a)
    t = R64(a * LOG2E64)
    n = mpmath.nint(t)
    f = t - n
    u = R64(mpf(2) ** abs(f) - 1)
    w = R64(1 + u)
    if f < 0:
        w = R64(1 / w)
    return w * mpf(2) ** n


def ln64(x):
    """FLDLN2; FYL2X: RN64(RN64(ln 2) * log2 x), unstored."""
    x = mpf(x)
    return R64(LN2_64 * mpmath.log(x) / mpmath.ln(2))


def ln_x87(x) -> float:
    return R53(ln64(x))


# ---------------------------------------------------------------- scoring
def within15(m: float, e: float) -> bool:
    if m == e:
        return True
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return False
    half = 0.5 * 10.0 ** (math.floor(math.log10(abs(e))) - 14)
    return abs(m - e) <= half


def ulpd(m: float, e: float) -> float:
    if m == e:
        return 0.0
    if not math.isfinite(m):
        return math.inf
    return abs(m - e) / math.ulp(e)


def score(models, rows, idx=None):
    idx = range(len(rows)) if idx is None else idx
    n = ex = w4 = w15 = 0
    for i in idx:
        m, e = models[i], rows[i]["excel"]
        n += 1
        ex += m == e
        w4 += ulpd(m, e) <= 4
        w15 += within15(m, e)
    return n, ex, w4, w15


def fmt(s):
    n, ex, w4, w15 = s
    return f"{ex:4d}/{n:<4d} w4 {w4:4d} w15 {w15:4d}"
