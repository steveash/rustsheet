"""T-3478 fresh sweeps (decision 0188 R4.2) over the domains RATE's and IRR's secant constants and the
F density's small-shape constant serve, past their FITTED ranges. Writes three probes-numeric cases
and a math TSV per case (50-digit mpmath, dps 60 checked at dps 80).

Run: cd tools && uv run python -I ../docs/research/t3478/gen_sweeps.py <repo>
"""

from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

import mpmath

from rustsheet_tools.accuracy.defs.annuity import rate as rate_math
from rustsheet_tools.accuracy.literal import round15, text
from rustsheet_tools.accuracy.model import DomainError, NoValueError
from rustsheet_tools.accuracy.sampler import log_uniform

REPO = Path(sys.argv[1])
OUT = REPO / "eval" / "suites" / "probes-numeric" / "cases"
PER = 50


def lit(x: float) -> str:
    return text(round15(x))


def converged(fn):
    vals = []
    for dps in (60, 80):
        with mpmath.workdps(dps):
            vals.append(fn())
    a, b = vals
    if a is None or b is None:
        return None
    if b != 0 and abs((a - b) / b) > mpmath.mpf("1e-45"):
        return None
    with mpmath.workdps(80):
        return mpmath.nstr(b, 50, strip_zeros=False)


# ------------------------------------------------------------------ RATE
def rate_point(rng: random.Random, stratum: int):
    def typical_rate():
        return (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-6, 0.5)

    r, nper = typical_rate(), rng.randint(1, 1000)
    lo_amt, hi_amt = 1e-2, 1e6
    guess = None
    if stratum == 0:
        r = -log_uniform(rng, 0.75, 0.999)
    elif stratum == 1:
        r = log_uniform(rng, 2, 100)
    elif stratum == 2:
        nper = round(log_uniform(rng, 1000, 1e5))
        r = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-8, 0.3)
    elif stratum == 3:
        lo_amt, hi_amt = (1e-8, 1e-4) if rng.random() < 0.5 else (1e8, 1e12)
    elif stratum == 4:
        guess = -log_uniform(rng, 0.5, 0.999) if rng.random() < 0.5 else log_uniform(rng, 10, 1000)
    else:
        r = rng.choice([-log_uniform(rng, 0.75, 0.999), log_uniform(rng, 2, 100), typical_rate()])
        nper = round(log_uniform(rng, 1, 1e5))
        lo_amt, hi_amt = 1e-8, 1e12
        guess = -log_uniform(rng, 0.5, 0.999) if rng.random() < 0.5 else log_uniform(rng, 10, 1000)
    if guess is None:
        guess = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-4, 0.5)
    if abs(nper * math.log1p(r)) > 600:
        return None
    sign = 1.0 if rng.random() < 0.5 else -1.0
    pv = sign * log_uniform(rng, lo_amt, hi_amt)
    fv = 0.0 if rng.random() < 0.5 else -sign * log_uniform(rng, lo_amt, hi_amt)
    due = rng.random() < 0.5
    with mpmath.workdps(40):
        x = mpmath.mpf(1) + r
        disc = x ** (-nper)
        pmt = float(-(pv + fv * disc) * r / ((1 + r * (1 if due else 0)) * (1 - disc)))
    if not math.isfinite(pmt) or pmt == 0 or abs(pmt) > 1e300:
        return None
    args = [nper, round15(pmt), round15(pv), round15(fv), 1 if due else 0]
    try:
        m = converged(
            lambda: rate_math(nper, mpmath.mpf(args[1]), mpmath.mpf(args[2]), mpmath.mpf(args[3]), due)
        )
    except NoValueError:
        return None
    except DomainError:
        m = "#NUM!"
    if m is None:
        return None
    f = "=RATE({},{},{},{},{},{})".format(
        text(nper), lit(args[1]), lit(args[2]), lit(args[3]), args[4], lit(guess)
    )
    names = [
        "rate -0.999..-0.75",
        "rate 2..100",
        "nper 1000..1E+5",
        "|pv|, |fv| 1E-8..1E-4 or 1E+8..1E+12",
        "guess -0.999..-0.5 or 10..1000",
        "all axes wide",
    ]
    return f, f"rate/{names[stratum]}", m


# ------------------------------------------------------------------ IRR
def irr_math(values: list[float]):
    def fn():
        cs = [mpmath.mpf(c) for c in values]

        def npv(v):
            s = mpmath.mpf(0)
            for c in reversed(cs):
                s = s * v + c
            return s

        # one sign change: unique positive v; bracket on (0, inf) in ln v
        lo, hi = mpmath.mpf(-1), mpmath.mpf(1)

        def f(u):
            return npv(mpmath.exp(u))

        for _ in range(60):
            if (f(lo) > 0) != (f(hi) > 0):
                break
            lo, hi = lo * 2, hi * 2
        else:
            return None
        for _ in range(400):
            mid = (lo + hi) / 2
            if (f(mid) > 0) == (f(lo) > 0):
                lo = mid
            else:
                hi = mid
            if hi - lo < mpmath.mpf(2) ** (-mpmath.mp.prec - 4):
                break
        u = (lo + hi) / 2
        return mpmath.expm1(-u)

    return converged(fn)


def irr_point(rng: random.Random, stratum: int):
    n = rng.randint(2, 41)
    scale = log_uniform(rng, 1e-6, 1e9)
    root = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-4, 0.99 if rng.random() < 0.3 else 2)
    guess = None
    if stratum == 0:
        n = round(log_uniform(rng, 42, 150))
    elif stratum == 1:
        scale = log_uniform(rng, 1e-12, 1e-6) if rng.random() < 0.5 else log_uniform(rng, 1e9, 1e15)
    elif stratum == 2:
        root = -log_uniform(rng, 0.99, 0.9999)
        n = rng.randint(2, 12)
    elif stratum == 3:
        root = log_uniform(rng, 99, 1e6)
        n = rng.randint(2, 6)
    elif stratum == 4:
        guess = -log_uniform(rng, 0.99, 0.9999) if rng.random() < 0.5 else log_uniform(rng, 100, 1e8)
    else:  # every axis outside at once
        n = round(log_uniform(rng, 2, 150))
        scale = log_uniform(rng, 1e-12, 1e15)
        root = (
            log_uniform(rng, 99, 1e4)
            if rng.random() < 0.3
            else ((1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-4, 0.999))
        )
        guess = -log_uniform(rng, 0.99, 0.9999) if rng.random() < 0.5 else log_uniform(rng, 100, 1e8)
    if guess is None and rng.random() < 0.5:
        guess = (1 if rng.random() < 0.7 else -1) * log_uniform(rng, 1e-3, 0.9)
    # conventional flows: one outlay then inflows; c0 set so NPV(root) = 0
    tail = [scale * rng.uniform(0.05, 1) for _ in range(n - 1)]
    with mpmath.workdps(40):
        v = 1 / (mpmath.mpf(1) + root)
        c0 = -sum(mpmath.mpf(c) * v ** (i + 1) for i, c in enumerate(tail))
        c0 = float(c0)
    if not math.isfinite(c0) or c0 == 0 or abs(c0) > 1e300 or abs(c0) < 1e-300:
        return None
    flows = [c0] + tail
    if rng.random() < 0.3:
        flows = [-c for c in flows]
    flows = [round15(c) for c in flows]
    m = irr_math(flows)
    if m is None:
        return None
    body = "{" + ",".join(lit(c) for c in flows) + "}"
    f = f"=IRR({body})" if guess is None else f"=IRR({body},{lit(guess)})"
    if len(f) > 8000:
        return None
    names = [
        "flows 42..150",
        "|c| 1E-12..1E-6 or 1E+9..1E+15",
        "root -0.9999..-0.99",
        "root 99..1E+6",
        "guess -0.9999..-0.99 or 100..1E+8",
        "all axes wide",
    ]
    return f, f"irr/{names[stratum]}", m


# ------------------------------------------------------------------ F density
def f_pdf(x: float, d1: int, d2: int):
    def fn():
        X, m, n = mpmath.mpf(x), mpmath.mpf(d1), mpmath.mpf(d2)
        a, b = m / 2, n / 2
        ln = (
            a * mpmath.log(m / n)
            + (a - 1) * mpmath.log(X)
            - (a + b) * mpmath.log1p(m * X / n)
            - mpmath.log(mpmath.beta(a, b))
        )
        return mpmath.exp(ln)

    return converged(fn)


def f_point(rng: random.Random, stratum: int):
    def dfs():
        if stratum in (0, 1):  # 11 <= min(d) <= 15
            lo = rng.randint(11, 15)
            hi = rng.randint(lo, 15) if stratum == 0 else round(log_uniform(rng, 16, 1e10))
        elif stratum in (2, 3):  # min(d) 1..10, max(d) 11..100
            lo = rng.randint(1, 10)
            hi = round(log_uniform(rng, 11, 100))
        else:  # min(d) 1..10, max(d) 100..1e6
            lo = rng.randint(1, 10)
            hi = round(log_uniform(rng, 100, 1e10))
        return (lo, hi) if rng.random() < 0.5 else (hi, lo)

    d1, d2 = dfs()
    if d1 == 1 and d2 <= 30:
        return None
    # x around the bulk (|P| small) on even strata, wider on odd ones
    mode = 1.0
    x = mode * (log_uniform(rng, 0.2, 5) if stratum % 2 == 0 else log_uniform(rng, 1e-3, 1e3))
    x = round15(x)
    m = f_pdf(x, d1, d2)
    if m is None or float(m) < 1e-300:
        return None
    f = f"=F.DIST({lit(x)},{d1},{d2},FALSE)"
    names = [
        "min(d) 11..15, max(d) <= 15",
        "min(d) 11..15, max(d) 16..1E+10",
        "min(d) 1..10, max(d) 11..100",
        "min(d) 1..10, max(d) 11..100, x wide",
        "min(d) 1..10, max(d) 100..1E+10",
        "min(d) 1..10, max(d) 100..1E+10, x wide",
    ]
    return f, f"fdensity/{names[stratum]}", m


def build(name: str, desc: str, tags: list[str], point, strata: int) -> None:
    rng = random.Random(f"T-3478/{name}/r4-fresh")
    cells, hyp, math_rows = {}, {}, []
    row = 1
    for s in range(strata):
        got = 0
        tries = 0
        while got < PER and tries < 5000:
            tries += 1
            p = point(rng, s)
            if p is None:
                continue
            f, tag, m = p
            cells[f"A{row}"] = f
            hyp[f"Calc!A{row}"] = tag
            math_rows.append(f"Calc!A{row}\t{m}\t{f}")
            row += 1
            got += 1
    case = {
        "format": "rustsheet-case/v1",
        "description": desc,
        "tags": tags,
        "expected": {},
        "hypotheses": hyp,
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    (OUT / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (OUT / f"{name}.math.tsv").write_text("cell\tmath\tformula\n" + "\n".join(math_rows) + "\n")
    print(name, row - 1)


build(
    "t3478-rate-r4-wide",
    "T-3478 fresh sweep (decision 0188 R4.2) of RATE past the secant fit's tagged range: one axis outside the tag per stratum (rate -0.999..-0.75, rate 2..100, nper 1000..1E+5, |pv| and |fv| 1E-8..1E-4 or 1E+8..1E+12, guess -0.999..-0.5 or 10..1000) and all at once; 50 each. Math: math.tsv (50-digit mpmath roots).",
    ["M9", "probe", "financial", "T-3478", "sweep", "rate", "r4"],
    rate_point,
    6,
)
build(
    "t3478-irr-r4-wide",
    "T-3478 fresh sweep (decision 0188 R4.2) of IRR past the secant fit's tagged range: conventional flows (one sign change), one axis outside the tag per stratum (flows, |c|, root low, root high, guess) and all at once; 50 each. Math: math.tsv (50-digit mpmath roots).",
    ["M9", "probe", "financial", "T-3478", "sweep", "irr", "r4"],
    irr_point,
    6,
)
build(
    "t3478-f-density-r4-wide",
    "T-3478 fresh sweep (decision 0188 R4.2) of F.DIST(x,d1,d2,FALSE) on shapes below LARGE_DF_MIN outside SMALL_SHAPE_OFFSET's tagged range (df 1..10, d1 1 with d2 up to 30): 11 <= min(d) <= 15 with max(d) up to 1E+10, and min(d) <= 10 with max(d) 11..1E+10; x near the bulk or wide; 50 each. Math: math.tsv (50-digit mpmath).",
    ["M9", "probe", "statistical", "T-3478", "sweep", "f-dist", "r4"],
    f_point,
    6,
)
