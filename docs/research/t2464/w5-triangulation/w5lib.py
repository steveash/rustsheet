"""T-2464 w5-triangulation shared helpers: cells, exact I_x (cached), Excel EXP/LN (cached), scoring."""
from __future__ import annotations

import csv, math, os, pickle, sys  # noqa: E401
from pathlib import Path

import mpmath
import numpy as np

T2464 = Path(__file__).resolve().parent.parent
HERE = T2464 / "w5-triangulation"
sys.path.insert(0, str(T2464))
import excelmath as em  # noqa: E402

mpmath.mp.dps = 40
LD = np.longdouble


def cells(fns=None):
    out = []
    for r in csv.DictReader(open(T2464 / "cells.tsv"), delimiter="\t"):
        if fns and r["fn"] not in fns:
            continue
        args = []
        for s in r["args"].split("|"):
            args.append(True if s == "True" else False if s == "False" else float(s))
        try:
            ex = float(r["excel"])
        except ValueError:
            continue
        out.append(dict(fn=r["fn"], case=r["case"], cell=r["cell"], args=args, excel=ex))
    return out


# ---------------------------------------------------------------- exact I_x
_EXF = HERE / "exact_cache.pkl"
_EX: dict = {}
if _EXF.exists():
    try:
        _EX = pickle.load(open(_EXF, "rb"))
    except Exception:
        _EX = {}


def save_cache():
    pickle.dump(_EX, open(_EXF, "wb"))


def ibeta(x, a, b, upper=False):
    """Exact regularized I_x(a,b) (upper: 1 - I_x = I_{1-x}(b,a) with exact 1-x), as mpf."""
    k = (x, a, b, upper)
    v = _EX.get(k)
    if v is None:
        v = mpmath.betainc(a, b, 0, x, regularized=True) if not upper else mpmath.betainc(a, b, x, 1, regularized=True)
        _EX[k] = v
    return v


def ibeta_xy(x, y, a, b):
    """I evaluated at an x whose complement is the given y (both doubles, x + y need not be 1): use the
    smaller one as the exact argument: if x <= y return I_x(a,b) else 1 - I_y(b,a)."""
    if x <= y:
        return ibeta(x, a, b)
    return 1 - ibeta(y, b, a)


# ---------------------------------------------------------------- Excel elementary functions (cached)
_XF = HERE / "xl_cache.pkl"
_XL: dict = {}
if _XF.exists():
    try:
        _XL = pickle.load(open(_XF, "rb"))
    except Exception:
        _XL = {}


def xl(name, xs):
    xs = [float(v) for v in xs]
    need = sorted({v for v in xs if (name, v) not in _XL})
    if need:
        for i in range(0, len(need), 200000):
            chunk = need[i:i + 200000]
            res = em.excel(name, chunk)
            for v, r in zip(chunk, res):
                _XL[(name, v)] = r
        pickle.dump(_XL, open(_XF, "wb"))
    return [_XL[(name, v)] for v in xs]


def EXP(v):
    return xl("EXP", [v])[0]


def LN(v):
    return xl("LN", [v])[0]


# ---------------------------------------------------------------- x87 stored ops
def x87(op, a, b):
    A, B = LD(a), LD(b)
    r = {"+": A + B, "-": A - B, "*": A * B, "/": A / B}[op]
    return float(r)


# ---------------------------------------------------------------- scoring
def ulpd(got, want):
    if got == want:
        return 0.0
    if want == 0 or not math.isfinite(got):
        return math.inf
    return abs(got - want) / math.ulp(want)


def w15(got, want):
    if got == want:
        return True
    if want == 0:
        return False
    e = math.floor(math.log10(abs(want)))
    return abs(got - want) <= 0.5 * 10 ** (e - 14)


def score(pairs):
    """pairs of (model, excel) -> (n, exact, within4, w15)."""
    n = e = f = w = 0
    for m, x in pairs:
        n += 1
        if m is None:
            continue
        d = ulpd(m, x)
        e += d == 0
        f += d <= 4
        w += w15(m, x)
    return n, e, f, w


def fmt(s):
    n, e, f, w = s
    return f"{e}/{f}/{w} of {n}"


def holdout(case, cell):
    import hashlib
    return int(hashlib.md5((case + cell).encode()).hexdigest(), 16) & 1
