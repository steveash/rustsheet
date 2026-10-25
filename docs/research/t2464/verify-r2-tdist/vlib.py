"""verify-r2-tdist: independent re-implementation of r2-tdist's bgrat_half_ideal model and scorer.

Nothing from r2-tdist is imported. Ingredients:
  Excel LN: ../excelmath.py (shared harness; engine's bit-exact LN model)
  Excel ERFC.PRECISE: the engine via the python bindings (own call)
  Excel log1p: re-implemented here from r2-log1p's docstring (x87 stored atanh series); cross-checked separately.
  exact I: mpmath.
Run with /home/user/rustsheet/bindings/python/.venv/bin/python.
"""

from __future__ import annotations

import csv
import hashlib
import math
import pickle
import sys
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T = HERE.parent
sys.path.insert(0, str(T))
import excelmath  # noqa: E402

mpf = mpmath.mpf
CACHE_F = HERE / "vcache.pkl"
C: dict = pickle.loads(CACHE_F.read_bytes()) if CACHE_F.exists() else {}


def save():
    CACHE_F.write_bytes(pickle.dumps(C))


# ---------- x87 stored op and Excel log1p (own implementation) ----------
def _rb(v, bits):
    return excelmath.round_bits(v, bits)


def X(op, a, b):
    A, B = mpf(a), mpf(b)
    r = {"+": A + B, "*": A * B, "/": A / B}[op]
    return float(_rb(_rb(r, 64), 53))


def log1p_xl(a: float) -> float:
    r = X("/", a, a + 2.0)
    t2 = X("*", r, r)
    p, s = r, None
    for k in range(1, 15):
        p = X("*", p, t2)
        tm = X("*", p, 1.0 / (2 * k + 1))
        s = tm if s is None else X("+", s, tm)
    return 2 * X("+", r, s)


# ---------- Excel functions ----------
def LN(xs):
    todo = sorted({x for x in xs if ("LN", x) not in C})
    if todo:
        for x, v in zip(todo, excelmath.excel("LN", todo)):
            C[("LN", x)] = v
    return [C[("LN", x)] for x in xs]


def ERFC(xs):
    import rustsheet as ic

    todo = sorted({x for x in xs if ("ERFC", x) not in C})
    for k in range(0, len(todo), 10000):
        ch = todo[k : k + 10000]
        m = ic.create("v", "en", "UTC")
        for i, x in enumerate(ch):
            m.update_cell_with_number(0, i + 1, 1, x)
            m.update_cell_with_formula(0, i + 1, 2, f"=ERFC.PRECISE(A{i + 1})")
        m.evaluate()
        for i, x in enumerate(ch):
            v = m.get_cell_value(0, i + 1, 2)
            C[("ERFC", x)] = float(v[-1] if isinstance(v, tuple) else v)
    return [C[("ERFC", x)] for x in xs]


# ---------- exact pieces ----------
def I_at_ln(a: float, lnx: float):
    """exact I_x(a, 1/2) with ln x = lnx (the double), as upper integral of the complement."""
    k = ("F", a, lnx)
    if k not in C:
        with mpmath.workdps(60):
            w = -mpmath.expm1(mpf(lnx))  # 1 - x exactly
            v = mpmath.betainc(0.5, a, w, 1, regularized=True)
            if v < mpf(10) ** -30:  # the upper integral cancels: use the direct lower integral
                with mpmath.workdps(80):
                    v = mpmath.betainc(a, 0.5, 0, mpmath.exp(mpf(lnx)), regularized=True)
            C[k] = v
    return C[k]


def I_exact(a: float, y: float):
    k = ("IE", a, y)
    if k not in C:
        with mpmath.workdps(60):
            v = mpmath.betainc(0.5, a, 1 - mpf(y), 1, regularized=True)
            if v < mpf(10) ** -30:
                with mpmath.workdps(80):
                    v = mpmath.betainc(a, 0.5, 0, mpf(y), regularized=True)
            C[k] = v
    return C[k]


# ---------- T wrapper ----------
def targs(t, n):
    tt = t * t
    u = n + tt
    return n / u, tt / u


def branch(n, y, z):
    """TOMS bratio dispatch for I_y(n/2, 1/2) with complement z."""
    a = n / 2
    if y <= 0.5:
        return "bpser"  # no swap, b0 = 1/2 <= 1
    # swap: a0 = 1/2, b0 = a, x0 = z
    if a <= 1:
        return "bpser_sw"
    if z >= 0.3:
        return "w1_bpser"
    if z < 0.1 and (z * a) ** 0.5 <= 0.7:
        return "bpser_sw"
    return "bgrat" if a > 15 else "bup+bgrat"


def model_I(n, y, z, cut=0.01, lnmode="cut", zround=True, erfc="xl"):
    a = n / 2
    nu = a - 0.25
    if lnmode == "cut":
        lnx = C[("LN", y)] if z > cut else log1p_xl(-z)
    elif lnmode == "LN":
        lnx = C[("LN", y)]
    zz = -nu * lnx
    with mpmath.workdps(60):
        zx = -mpf(nu) * mpf(lnx)
        qx = mpmath.erfc(mpmath.sqrt(zx))
        s = math.sqrt(zz) if zround else float(mpmath.sqrt(zx))
        if erfc == "xl":
            q = mpf(C[("ERFC", s)])
        else:
            q = mpmath.erfc(mpf(s))
        return float(I_at_ln(a, lnx) * q / qx)


def needed_s(n, y, z, cut=0.01, zround=True):
    a = n / 2
    nu = a - 0.25
    lnx = C[("LN", y)] if z > cut else log1p_xl(-z)
    if zround:
        return math.sqrt(-nu * lnx)
    with mpmath.workdps(60):
        return float(mpmath.sqrt(-mpf(nu) * mpf(lnx)))


# ---------- scoring ----------
def w15(m, e):
    if m == e:
        return True
    if e == 0 or not math.isfinite(m):
        return False
    return abs(m - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14)


def score(pairs):
    n = len(pairs)
    ex = sum(m == e for m, e in pairs)
    u4 = sum(m == e or (math.isfinite(m) and abs(m - e) <= 4 * math.ulp(e)) for m, e in pairs)
    v = sum(w15(m, e) for m, e in pairs)
    return f"{ex}/{u4}/{v} of {n}"


def myhold(key: str) -> bool:
    return hashlib.sha256(("verify" + key).encode()).digest()[0] & 1 == 1


def load_cells():
    """cells.tsv T tails, L side: (fn, case, cell, t, n, scale, excel)."""
    out = []
    for r in csv.DictReader(open(T / "cells.tsv"), delimiter="\t"):
        fn = r["fn"]
        if fn not in ("T.DIST.RT", "T.DIST.2T", "TDIST", "T.DIST"):
            continue
        a = r["args"].split("|")
        try:
            t, df, e = float(a[0]), float(a[1]), float(r["excel"])
        except ValueError:
            continue
        if not math.isfinite(e) or e == 0:
            continue
        n = float(math.trunc(df))
        if n < 1:
            continue
        if fn == "T.DIST.RT":
            if t <= 0:
                continue
            sc = 0.5
        elif fn == "T.DIST.2T":
            if t <= 0:
                continue
            sc = 1.0
        elif fn == "TDIST":
            tails = 1 if a[2] == "True" else 0 if a[2] == "False" else math.trunc(float(a[2]))
            if t <= 0:
                continue
            sc = 1.0 if tails == 2 else 0.5
        else:
            if a[2] != "True" or t >= 0:
                continue
            sc = 0.5
        out.append(dict(fn=fn, case=r["case"], cell=r["cell"], t=abs(t), n=n, sc=sc, excel=e))
    return out
