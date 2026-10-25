"""T-2464 w3-series shared helpers: data, exact values, Excel elementary functions (cached), x87 vector ops, scoring.

x87 stored ops are numpy longdouble (80-bit, 64-bit significand on x86-64 Linux) converted to float64 after each
op: fl53(fl64(a op b)), checked against excelmath.x87 on 80,000 random ops (0 differences).
"""

from __future__ import annotations

import csv
import math
import os
import pickle
import sys
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
sys.path.insert(0, str(T2464))
import excelmath as em  # noqa: E402

LD = np.longdouble
fl64, fl53 = em.fl64, em.fl53
U = 2.0**-53
mpmath.mp.prec = 200


# ---------------------------------------------------------------- data
def load(cum=None, branch=None):
    """All BETA.DIST rows with A=0,B=1 and 0<x<1: beta_exact.tsv (src 'be') and w2's probes (src 'w2')."""
    rows = []
    for r in csv.DictReader(open(T2464 / "beta_exact.tsv"), delimiter="\t"):
        rows.append(
            dict(
                src="be",
                case=r["case"].split("/")[-1].replace(".case.json", ""),
                cell=r["cell"],
                x=float(r["x"]),
                a=float(r["a"]),
                b=float(r["b"]),
                cum=r["cum"] == "1",
                excel=float(r["excel"]),
                exact=None,
            )
        )
    for r in csv.DictReader(open(T2464 / "w2-forensics" / "exact_w2f.tsv"), delimiter="\t"):
        x = float(r["x"])
        if not (0 < x < 1):
            continue
        rows.append(
            dict(
                src="w2",
                case=r["case"],
                cell=r["cell"],
                x=x,
                a=float(r["a"]),
                b=float(r["b"]),
                cum=r["cum"] == "1",
                excel=float(r["excel"]),
                exact=r["exact"],
            )
        )
    for r in rows:
        r["branch"] = "direct" if r["x"] <= r["a"] / (r["a"] + r["b"]) else "reflected"
        if r["src"] == "w2":
            i = int(r["cell"].split("!A")[1]) - 1
            r["block"] = f"{r['case'].replace('t2464-w2f-', '')}:{i // (1024 if 'absweep' in r['case'] else 2048)}"
        else:
            r["block"] = "be"
    if cum is not None:
        rows = [r for r in rows if r["cum"] == cum]
    if branch is not None:
        rows = [r for r in rows if r["branch"] == branch]
    return rows


_EX = {}
_EXF = HERE / "exact_cache.pkl"
if _EXF.exists():
    _EX = pickle.load(open(_EXF, "rb"))


def exact_cdf(x, a, b):
    k = ("cdf", x, a, b)
    if k not in _EX:
        with mpmath.workprec(200):
            _EX[k] = mpmath.betainc(mpmath.mpf(a), mpmath.mpf(b), 0, mpmath.mpf(x), regularized=True)
    return _EX[k]


def exact_pdf(x, a, b):
    k = ("pdf", x, a, b)
    if k not in _EX:
        X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
        _EX[k] = X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B)
    return _EX[k]


def save_exact():
    pickle.dump(_EX, open(_EXF, "wb"))


# ---------------------------------------------------------------- Excel elementary functions (cached)
_XL = {}
_XLF = HERE / "xl_cache.pkl"
if _XLF.exists():
    _XL = pickle.load(open(_XLF, "rb"))


def xl(name, xs):
    """Excel's NAME(x) (EXP, LN, LOG10, GAMMALN, GAMMA) for an array of doubles, memoized."""
    xs = np.asarray(xs, dtype=np.float64)
    need = sorted({float(v) for v in xs.ravel() if (name, float(v)) not in _XL and math.isfinite(v)})
    if need:
        for i in range(0, len(need), 200000):
            ch = need[i : i + 200000]
            for v, r in zip(ch, em.excel(name, ch)):
                _XL[(name, v)] = r
        pickle.dump(_XL, open(_XLF, "wb"))
    return np.array([_XL.get((name, float(v)), math.nan) for v in xs.ravel()]).reshape(xs.shape)


# ---------------------------------------------------------------- x87 stored ops on arrays
def s(v):
    """store an extended value to double"""
    return np.asarray(v, dtype=LD).astype(np.float64)


def add(a, b):
    return (np.asarray(a, dtype=LD) + np.asarray(b, dtype=LD)).astype(np.float64)


def sub(a, b):
    return (np.asarray(a, dtype=LD) - np.asarray(b, dtype=LD)).astype(np.float64)


def mul(a, b):
    return (np.asarray(a, dtype=LD) * np.asarray(b, dtype=LD)).astype(np.float64)


def div(a, b):
    return (np.asarray(a, dtype=LD) / np.asarray(b, dtype=LD)).astype(np.float64)


# ---------------------------------------------------------------- scoring
def digits15_ok(got, want):
    """|want - got| <= half a unit in the 15th significant digit of want (excel)."""
    if got == want:
        return True
    if not math.isfinite(got) or want == 0:
        return False
    e = math.floor(math.log10(abs(want)))
    return abs(got - want) <= 0.5 * 10.0 ** (e - 14)


def ulpd(got, want):
    if got == want:
        return 0.0
    if not math.isfinite(got):
        return math.inf
    return abs(got - want) / math.ulp(want)


def score(model, excel):
    """(bit-exact, within 4 ulp, within 15 digits, n)"""
    ex = w4 = d15 = 0
    for m, e in zip(model, excel):
        m = float(m)
        u = ulpd(m, e)
        ex += u == 0
        w4 += u <= 4
        d15 += digits15_ok(m, e)
    return ex, w4, d15, len(excel)


def fmt(sc):
    return f"{sc[0]}/{sc[1]}/{sc[2]} of {sc[3]}"


def rel_units(vals, exacts):
    return np.array([float((mpmath.mpf(float(v)) - e) / e) / U for v, e in zip(vals, exacts)])


def split_half(rows):
    """deterministic hold-out split: tune on even hash, hold out odd."""
    import hashlib

    tune, hold = [], []
    for r in rows:
        h = hashlib.md5(f"{r['case']}{r['cell']}".encode()).digest()[0]
        (tune if h % 2 == 0 else hold).append(r)
    return tune, hold
