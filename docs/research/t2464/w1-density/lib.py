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

mpmath.mp.prec = 160
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


_EXPC: dict = {}


def exp_v6(a) -> float:
    """Excel's EXP model V6 for an mpf input a (a double or an extended value), memoised."""
    a = mpf(a)
    key = a._mpf_
    v = _EXPC.get(key)
    if v is not None:
        return v
    v = _exp_v6(a)
    _EXPC[key] = v
    return v


def _exp_v6(a) -> float:
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


def exp2_v6_ext(t):
    """2^t as the V6 tail does it for an extended t (FRNDINT, F2XM1, 1+u, reciprocal for f<0, FSCALE)."""
    t = mpf(t)
    n = mpmath.nint(t)
    f = t - n
    u = R64(mpf(2) ** abs(f) - 1)
    w = R64(1 + u)
    if f < 0:
        w = R64(1 / w)
    return w * mpf(2) ** n


def exp2_v6(t) -> float:
    return R53(exp2_v6_ext(t))


def fyl2x(y, x):
    """FYL2X: RN64(y * log2 x), y and x as given (extended values allowed)."""
    return R64(mpf(y) * mpmath.log(mpf(x)) / mpmath.ln(2))


def fyl2xp1(y, x):
    """FYL2XP1: RN64(y * log2(1 + x))."""
    return R64(mpf(y) * mpmath.log1p(mpf(x)) / mpmath.ln(2))


def load_probe(name="probe1"):
    """Rows of a w1-density probe: x, a, b, excel, exact (correctly rounded density), sheet, k."""
    import json

    meta = json.load(open(HERE / "probe" / f"{name}.meta.json"))
    out = json.load(open(HERE / "probe" / f"{name}.out.json"))["sheets"]
    cache = HERE / "probe" / f"{name}.rows.pkl"
    if cache.exists():
        return pickle.loads(cache.read_bytes())
    rows = []
    old = mpmath.mp.prec
    mpmath.mp.prec = 160
    for j, (a, b) in enumerate(meta["conf"]):
        sh = out[f"C{j}"]
        for k, x in enumerate(meta["xs"]):
            c = sh.get(f"B{k+1}")
            if not c or c.get("t") != "n":
                continue
            X_, A_, B_ = mpf(x), mpf(a), mpf(b)
            ex = X_ ** (A_ - 1) * (1 - X_) ** (B_ - 1) / mpmath.beta(A_, B_)
            rows.append(dict(case=f"{name}C{j}", cell=f"B{k+1}", k=k, x=x, a=a, b=b, excel=float(c["v"]), exact=float(ex)))
    mpmath.mp.prec = old
    cache.write_bytes(pickle.dumps(rows))
    return rows


def pdf_region():
    """The pdf rows, their columns and Excel's GAMMALN pieces, restricted to the shape region argv[1]
    (default a<1,b<=8; 'all' for every row). Shared preamble of enum3-5 and interval1."""
    mpmath.mp.prec = 200
    rows = load_pdf()
    X = [r["x"] for r in rows]
    A = [r["a"] for r in rows]
    Bv = [r["b"] for r in rows]
    AB = [a + b for a, b in zip(A, Bv)]
    gl = xl("GAMMALN", A), xl("GAMMALN", Bv), xl("GAMMALN", AB)

    def region(i):
        a, b = A[i], Bv[i]
        ra = "a<1" if a < 1 else ("a<8" if a < 8 else "a>=8")
        rb = "b<1" if b < 1 else ("b<=8" if b <= 8 else "b>8")
        return ra + "," + rb

    want = sys.argv[1] if len(sys.argv) > 1 else "a<1,b<=8"
    idx = [i for i in range(len(rows)) if want == "all" or region(i) == want]
    return rows, X, A, Bv, AB, gl, want, idx


def model_table():
    """A fresh {model name: {row: value}} table and its put(name, row, value)."""
    models = {}

    def put(k, i, v):
        models.setdefault(k, {})[i] = v

    return models, put
