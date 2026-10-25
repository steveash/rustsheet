"""verify-r5-f-edges: independent scorer. Grades a model against Excel per cell:
bit-exact / within 4 ulp / within 15 digits (|excel - model| <= half a unit of excel's 15th significant digit).
An error or a 0 (either side) matches only when both are the same error or both 0.
Baselines: CR = correctly rounded exact density (mpmath, truncated df) with x87-free plain rules
(df trunc, ranges, x<0, x=0 limits) and a subnormal result flushed to 0."""
from __future__ import annotations

import math
import struct
import sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

import mpmath as mp

import vparse

HERE = Path(__file__).resolve().parent
mp.mp.prec = 300


def bits(v: float) -> int:
    return struct.unpack("<q", struct.pack("<d", v))[0]


def w15(model: float, excel: float) -> bool:
    e = Decimal(excel)
    exp10 = e.adjusted()  # excel = d.ddd x 10^exp10
    half = Fraction(5) * Fraction(10) ** (exp10 - 15)
    return abs(Fraction(model) - Fraction(excel)) <= half


def grade(model, excel):
    if isinstance(model, str) or isinstance(excel, str) or model == 0 or excel == 0:
        ok = (isinstance(model, str) and model == excel) or (
            not isinstance(model, str) and not isinstance(excel, str) and model == 0 and excel == 0)
        return ok, ok, ok
    if math.copysign(1, model) != math.copysign(1, excel):
        return False, False, False
    u = abs(bits(model) - bits(excel))
    return u == 0, u <= 4, w15(model, excel)


def tally(pairs):
    g = [grade(m, e) for m, e in pairs]
    return (sum(a for a, _, _ in g), sum(b for _, b, _ in g), sum(c for _, _, c in g), len(g))


def fmt(t):
    return f"{t[0]}/{t[1]}/{t[2]} of {t[3]}"


def exact_pdf(x: float, m: float, n: float):
    X = mp.mpf(x)
    a, b = mp.mpf(m) / 2, mp.mpf(n) / 2
    lg = mp.loggamma(a + b) - mp.loggamma(a) - mp.loggamma(b) + a * mp.log(mp.mpf(m) / n) \
        + (a - 1) * mp.log(X) - (a + b) * mp.log1p(mp.mpf(m) * X / n)
    return mp.e ** lg


def cr(x: float, d1: float, d2: float):
    m, n = float(math.trunc(d1)), float(math.trunc(d2))
    if m < 1 or n < 1 or m > 1e10 or n > 1e10 or x < 0:
        return "#NUM!"
    if x == 0:
        return "#NUM!" if m == 1 else (1.0 if m == 2 else 0.0)
    f = float(exact_pdf(x, m, n))
    return 0.0 if f < 2.0**-1022 else f


def model_edge():
    sys.path.insert(0, str(HERE.parent / "r5-f-edges"))
    import edgemodel  # noqa: E402  (checked against RESULTS.md section 1 before import)
    return edgemodel.value


def section(r) -> str:
    """My own sectioning, from the formula only (not the worker's labels)."""
    x, d1, d2 = r["x"], r["d1"], r["d2"]
    m, n = math.trunc(d1), math.trunc(d2)
    if r["twin"] is not None:
        return "twin"
    if x <= 0 or m < 1 or n < 1 or m > 1e10 or n > 1e10 or (0 < x < 2.0**-1022) or not math.isfinite(m * x):
        return "rule"
    if x > 1e300 and m * x > 1e307:
        return "ovf-near"
    if m != d1 or n != d2:
        return "frac"
    if m == 1:
        return "d1=1"
    if m == 2:
        return "d1=2"
    if n == 1:
        return "d2=1"
    if n == 2:
        return "d2=2"
    if x < 1e-6:
        return "tinyx"
    if x > 1e6:
        return "hugex"
    return "rest"


def run(case: str):
    rs = vparse.rows(case)
    ME = model_edge()
    by = {}
    for r in rs:
        me = ME(r["x"], r["d1"], r["d2"])
        c = cr(r["x"], r["d1"], r["d2"])
        r["me"], r["cr"] = me, c
        by.setdefault(section(r), []).append(r)
    print(f"# {case}: {len(rs)} rows (B; C/2^k where present)")
    allr = [r for v in by.values() for r in v]
    print(f"{'all':9s} M-edge {fmt(tally([(r['me'], r['excel']) for r in allr])):24s} CR {fmt(tally([(r['cr'], r['excel']) for r in allr]))}")
    for k in sorted(by):
        v = by[k]
        print(f"{k:9s} M-edge {fmt(tally([(r['me'], r['excel']) for r in v])):24s} CR {fmt(tally([(r['cr'], r['excel']) for r in v]))}")
    tw = [r for r in rs if r["twin"]]
    same = sum(1 for r in tw if (r["excel"] == r["twin"]["excel"]) and (isinstance(r["excel"], str) or bits(r["excel"]) == bits(r["twin"]["excel"])))
    print(f"twins B==D bitwise: {same}/{len(tw)}")
    # zeros / errors decisions
    dec = [r for r in rs if isinstance(r["excel"], str) or r["excel"] == 0 or isinstance(r["me"], str) or r["me"] == 0]
    okd = sum(grade(r["me"], r["excel"])[0] for r in dec)
    print(f"error/zero decisions (either side error or 0): M-edge {okd}/{len(dec)}")
    for r in dec:
        if not grade(r["me"], r["excel"])[0]:
            print("   miss", r["row"], r["fa"], r["d1"], r["d2"], "excel", r["excel"], "model", r["me"])
    return rs


if __name__ == "__main__":
    for c in sys.argv[1:]:
        run(c)
