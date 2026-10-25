"""T-2464 r2-log1p shared helpers: exact-double formula literals, probe case I/O, and the inversion of
BINOM.DIST(0, n, p, FALSE) = EXP(n * l) to Excel's stored l = ln(1 - p).

Reuses ../excelmath.py (Excel EXP/LN via the engine harness) and ../w1-density/lib.py (mpmath EXP V6/LN models)."""
from __future__ import annotations

import json
import math
import pickle
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE.parent / "w1-density"))
import mpmath  # noqa: E402

import excelmath  # noqa: E402

mpmath.mp.prec = 200
mpf = mpmath.mpf
SUITE = HERE.parents[3] / "eval" / "suites" / "probes-numeric" / "cases"


def lit(x: float) -> str:
    """An Excel formula fragment that evaluates exactly to the double x (two <= 27-bit integer pieces)."""
    if x == 0:
        return "0"
    m, e = math.frexp(x)  # x = m * 2^e, 0.5 <= |m| < 1
    M = int(m * 2**53)
    E = e - 53
    hi, lo = M >> 26, M & ((1 << 26) - 1)
    assert hi * 2.0 ** (E + 26) + lo * 2.0**E == x
    return f"({hi}*2^{E + 26}+{lo}*2^{E})"


def run(x0: float, n: int, below: int = 0) -> list[float]:
    """n consecutive doubles starting `below` ulps under x0."""
    x = x0
    for _ in range(below):
        x = math.nextafter(x, -math.inf)
    out = [x]
    for _ in range(n - 1):
        out.append(math.nextafter(out[-1], math.inf))
    return out


def binom_n(p: float) -> int:
    """Power of two n with n*|ln(1-p)| in [256, 512): n*l is exact and EXP(n*l) separates adjacent l."""
    l = -math.log1p(-p) if p < 0.9 else -math.log(1 - p)
    return 2 ** max(2, math.ceil(math.log2(256 / l)))


def write_case(name: str, desc: str, cols: dict[str, list[str]]):
    """cols: column letter -> list of formulas (row 1..)."""
    cells = {}
    for c, fs in cols.items():
        for i, f in enumerate(fs):
            cells[f"{c}{i + 1}"] = f
    assert len(cells) <= 16384, len(cells)
    case = {"format": "rustsheet-case/v1", "description": desc,
            "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta", "log1p"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    path = SUITE / f"t2464-r2-log1p-{name}.case.json"
    path.write_text(json.dumps(case, indent=1))
    return path


def read_oracle(name: str) -> dict[str, float]:
    p = SUITE / f"t2464-r2-log1p-{name}.oracle.json"
    if not p.exists():
        p = HERE / f"{name}.out.json"
    d = json.loads(p.read_text())
    return {k: (v.get("v") if v.get("t") == "n" else v.get("v", None)) for k, v in d["sheets"]["Calc"].items()}


_C = HERE / "xl_cache.pkl"
_cache: dict = pickle.loads(_C.read_bytes()) if _C.exists() else {}


def xl(name: str, xs):
    need = sorted({float(x) for x in xs if (name, float(x)) not in _cache and math.isfinite(x)})
    for i in range(0, len(need), 50000):
        ch = need[i:i + 50000]
        for x, v in zip(ch, excelmath.excel(name, ch)):
            _cache[(name, x)] = v
    if need:
        _C.write_bytes(pickle.dumps(_cache))
    return [_cache.get((name, float(x)), math.nan) for x in xs]


def cr_log1p(t: float) -> float:
    return float(mpmath.log1p(mpf(t)))


def nb(x: float, k: int) -> float:
    """x moved k ulps (k may be negative)."""
    d = math.inf if k > 0 else -math.inf
    for _ in range(abs(k)):
        x = math.nextafter(x, d)
    return x


def invert_binom(ps, ns, vals, width=6):
    """For each p: the set of doubles l within `width` ulps of CR log1p(-p) (or LN(1-p)) with EXP(n*l) == excel.
    Returns list of lists of l offsets in ulps (relative to CR log1p(-p)) and the CR value."""
    cands = []
    for p, n in zip(ps, ns):
        l0 = cr_log1p(-p)
        cands.append([nb(l0, k) for k in range(-width, width + 1)])
    allE = [l * n for cs, n in zip(cands, ns) for l in cs]
    ex = xl("EXP", allE)
    out, j = [], 0
    for cs, n, v in zip(cands, ns, vals):
        hits = []
        for k, l in enumerate(cs):
            if ex[j] == v:
                hits.append(k - width)
            j += 1
        out.append(hits)
    return out


def r_of(p: float, mode: str = "x") -> float:
    """r = a/(a+2), a = -p: 'x' = x87 stored fl53(fl64(a/fl(a+2))) (Excel, see RESULTS), 'd' = IEEE double."""
    a = -p
    s = a + 2.0  # exact or singly rounded either way (one op)
    if mode == "d":
        return a / s
    return excelmath.x87("/", a, s)
