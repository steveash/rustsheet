"""T-2586 r5-f-edges probes: the edges of F.DIST(x, d1, d2, FALSE).
Each row: A = x (an exact formula, echoed), B = F.DIST(A, d1, d2, FALSE); optional C = F.DIST(A, d1, d2, FALSE)*2^600
(a density below 1E-200, so a kept subnormal shows), D = F.DIST(A, trunc(d1), trunc(d2), FALSE) (non-integer df
twins). Sections (kind):
  zero    x = 0 for d1, d2 lists (T-2095's rule, df truncation at 0)
  neg     x < 0
  dflow   df below 1, at 1 - 2^-53, 1 + 2^-52, 2 - 2^-52 (with the truncated twin)
  dfhigh  df just above 1E10 (truncation before the cut)
  frac    non-integer df on runs of consecutive doubles, with the truncated twin
  tiny    x = 10^-k (k to 307) over small shapes, scaled column for tiny densities
  thresh  geometric x runs across the point where F*x underflows (the EXP(E)/x structure's fingerprint)
  sub     x a kept subnormal (EXP(-720), EXP(-740) inside the call) and x near 2^-1022
  huge    x = 10^k (k to 308), scaled column
  hthr    geometric x runs across the point where F becomes subnormal at huge x
  ovf     x at fl(DBL_MAX/m) +- k ulp (the d1*x overflow rule), non-integer d1 at the edge
  vrun    runs of 24 consecutive doubles at d1 or d2 in {1, 2} (hold-out only: new partner df)
Probes: a (in-sample) and hold (hold-out, other values; built and filled with a, read once by final_score.py).
Usage: edgeprobe.py [--write]"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import mpmath

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-fdensity"))
import flib as F  # noqa: E402

MP = F.MP
PREFIX = "t2586-r5-f-edges"
DMAX = sys.float_info.max


def enc(x: float) -> str:
    """Exact formula for a positive normal double x, with no subnormal intermediate: (H*2^26+L)*2^E[*2^E2]."""
    m, e = math.frexp(x)
    M, E = int(m * 2**53), e - 53
    H, L = M >> 26, M & ((1 << 26) - 1)
    core = f"({H}*2^26+{L})"
    if E >= -1000:
        return f"{core}*2^{E}"
    return f"{core}*2^-500*2^{E + 500}"


def geo(lo: float, hi: float, n: int) -> list[float]:
    return [float(mpmath.mpf(lo) * (mpmath.mpf(hi) / lo) ** (mpmath.mpf(i) / (n - 1))) for i in range(n)]


def nearest(v) -> float:
    return float(mpmath.mpf(v))


def row(kind, x, d1, d2, xf=None, d1s=None, d2s=None, scale=None, twin=False):
    """x: the double (None when the x formula xf is special); d1, d2: Excel's df values (floats)."""
    if xf is None:
        xf = "0" if x == 0 else enc(x)
    if scale is None and x and x > 0 and d1 >= 1 and d2 >= 1:
        scale = _lnf(x, math.trunc(d1), math.trunc(d2)) < -460  # density below ~1E-200
    return dict(
        kind=kind,
        x=x,
        xf=xf,
        d1=d1,
        d2=d2,
        d1s=d1s or f"{d1!r}".removesuffix(".0"),
        d2s=d2s or f"{d2!r}".removesuffix(".0"),
        scale=bool(scale),
        twin=twin,
    )


def _lnf(x, m, n):
    X, M, N = mpmath.mpf(x), mpmath.mpf(m), mpmath.mpf(n)
    return float(
        (M / 2) * mpmath.log(M * X / (M * X + N))
        + (N / 2) * mpmath.log(N / (M * X + N))
        - mpmath.log(X)
        - mpmath.log(mpmath.beta(M / 2, N / 2))
    )


def sections(name: str) -> list[dict]:
    H = name == "hold"
    out: list[dict] = []
    # zero
    d1z = (1.25, 2.75, 1, 2, 5, 3.0001) if H else (1, 1.5, 1.999999, 2, 2.5, 2.99999, 3, 3.5, 4, 10, 1e10)
    d2z = (3, 50.5) if H else (1, 2.5, 7, 1e10)
    out += [row("zero", 0.0, a, b) for a in d1z for b in d2z]
    # neg
    out += [
        row("neg", None, a, b, xf=f)
        for a, b in ((1, 1), (2, 7), (5, 5))
        for f in (("-3", "-(2^-900)") if H else ("-1", "-(2^-1000)"))
    ]
    # dflow / dfhigh
    lows = ("0", "0.75", "0.99999") if H else ("0", "0.5", "0.9999999", "-1")
    for s in lows:
        out += [row("dflow", 1.0, float(s), 5.0, d1s=s), row("dflow", 1.0, 5.0, float(s), d2s=s)]
    for s, v in (
        ("1-2^-53", 1 - 2**-53),
        ("1+2^-52", 1 + 2**-52),
        ("2-2^-52", 2 - 2**-52),
        ("3-2^-51", 3 - 2**-51),
    ):
        out += [
            row("dflow", 0.7 if H else 1.3, v, 5.0, d1s=s, twin=True),
            row("dflow", 0.7 if H else 1.3, 5.0, v, d2s=s, twin=True),
        ]
    for s in ("10000000000.25", "10000000000.75") if H else ("10000000000.5", "10000000000.9999"):
        out += [
            row("dfhigh", 2.0, float(s), 5.0, d1s=s, twin=True),
            row("dfhigh", 2.0, 5.0, float(s), d2s=s, twin=True),
        ]
    # frac
    fr = (
        [(1.75, 2.5), (2.25, 1.5), (4.5, 9.75), (30.5, 2.999)]
        if H
        else [
            (1.5, 1),
            (1, 1.5),
            (2.5, 3.25),
            (3.999, 7.5),
            (1.9999999, 2.9999999),
            (10.75, 20.5),
            (150.5, 99.9),
        ]
    )
    for a, b in fr:
        m, n = math.trunc(a), math.trunc(b)
        for t in (0.3, 3.1) if H else (0.2, 0.9, 4.0):
            out += [row("frac", x, a, b, twin=True) for x in MP.run(t * n / m * 1.0000371, 0, 16)]
    # tiny
    ks = (7, 33, 77, 133, 177, 222, 288, 305) if H else (5, 10, 20, 50, 100, 150, 200, 250, 300, 307)
    sh = (
        [(1, 3), (2, 3), (3, 3), (7, 2), (12, 4), (5, 40)]
        if H
        else [(m, n) for m in (1, 2, 3, 4, 5, 6, 8, 10, 20) for n in (1, 2, 5, 30)]
    )
    out += [row("tiny", nearest(mpmath.mpf(10) ** -k), m, n) for m, n in sh for k in ks]
    # thresh: F*x = E^... underflows near x where ln F + ln x = -708 .. -745
    for m, n in [(5, 3), (12, 7)] if H else [(3, 5), (4, 5), (10, 3), (6, 1)]:
        lo = _solve(lambda x: _lnf(x, m, n) + math.log(x) + 750, m, n)
        hi = _solve(lambda x: _lnf(x, m, n) + math.log(x) + 690, m, n)
        out += [row("thresh", x, m, n, scale=True) for x in geo(lo, hi, 40)]
    # sub
    for m in (1, 2, 4) if H else (1, 2, 3):
        for f in ("EXP(-730)",) if H else ("EXP(-720)", "EXP(-740)"):
            out.append(row("sub", None, m, 5.0, xf=f, scale=m >= 3))
    base = 2.0**-1022
    out += [
        row("sub", x, 2, n, scale=True)
        for n in ((3, 9) if H else (1, 5))
        for x in [base * (1 + j / 8) for j in range(8)] + [base * 2**j for j in (3, 6, 10)]
    ]
    # huge
    ksh = (
        (13, 77, 155, 201, 204.5, 206.5, 233, 290, 307.5)
        if H
        else (10, 50, 100, 150, 200, 203, 204, 205, 206, 207, 210, 215, 250, 300, 307, 308)
    )
    shh = [(1, 5), (3, 1), (7, 2), (2, 6)] if H else [(m, n) for m in (1, 2, 5, 50) for n in (1, 2, 3, 4, 10)]
    out += [row("huge", nearest(mpmath.mpf(10) ** k), m, n) for m, n in shh for k in ksh]
    # hthr: F subnormal at huge x
    for m, n in [(3, 1), (1, 3)] if H else [(1, 1), (4, 1), (1, 2), (2, 2)]:
        lo = _solve(lambda x: _lnf(x, m, n) + 690, m, n, big=True)
        hi = _solve(lambda x: _lnf(x, m, n) + 750, m, n, big=True)
        out += [row("hthr", x, m, n, scale=True) for x in geo(lo, min(hi, 1e307), 32)]
    # ovf
    for m in (5, 6) if H else (2, 3, 7, 10):
        x0 = nearest(mpmath.mpf(DMAX) / m)
        out += [row("ovf", MP.step(x0, k), m, 3.0) for k in range(-3, 4)]
    for a, xv in (
        ((1.25, DMAX), (2.5, nearest(mpmath.mpf(DMAX) / 2.5)), (3.75, nearest(mpmath.mpf(DMAX) / 3.5)))
        if H
        else (
            (1.5, DMAX),
            (1.99, DMAX),
            (2.5, nearest(mpmath.mpf(DMAX) / 2.25)),
            (3.5, nearest(mpmath.mpf(DMAX) / 3.25)),
        )
    ):
        out.append(row("ovf", xv, a, 3.0, twin=True))
    out += [row("ovf", DMAX, 1, n) for n in ((2, 7) if H else (1, 3, 1e10))]
    # vrun (hold-out only)
    if H:
        for m, n in [(1, 9), (1, 40), (2, 9), (2, 40), (9, 1), (40, 1), (9, 2), (40, 2), (1, 2), (2, 1)]:
            for t in (0.07, 0.55, 1.9, 13.0):
                out += [row("vrun", x, m, n) for x in MP.run(t * n / m * 1.0000913, 0, 24)]
    return out


def _solve(fn, m, n, big=False):
    """x with fn(x) = 0 by bisection on ln x (fn decreasing in x for small x, increasing for big when inverted)."""
    lo, hi = (0.0, 709.0) if big else (-708.0, 0.0)
    for _ in range(200):
        mid = (lo + hi) / 2
        v = fn(math.exp(mid))
        if (v > 0) != big:  # small x: fn grows with x (side 1, a >= 1.5); big: ln F falls with x
            hi = mid
        else:
            lo = mid
    return math.exp((lo + hi) / 2)


def cells_of(name: str) -> dict:
    c = {}
    for r, o in enumerate(sections(name), 1):
        A = f"A{r}"
        c[A] = "=" + o["xf"]
        call = f"F.DIST({A},{o['d1s']},{o['d2s']},FALSE)"
        c[f"B{r}"] = "=" + call
        if o["scale"]:
            c[f"C{r}"] = f"={call}*2^600"
        if o["twin"]:
            c[f"D{r}"] = f"=F.DIST({A},{math.trunc(o['d1'])},{math.trunc(o['d2'])},FALSE)"
    return c


def build(name):
    cells = cells_of(name)
    assert len(cells) <= 16384, len(cells)
    desc = (
        f"T-2586 r5-f-edges probe {name}: F.DIST(x,d1,d2,FALSE) at its edges (x = 0, x < 0, df < 1, df > 1E10, "
        "non-integer df twins, tiny and huge x with a *2^600 column, the d1*x overflow, d = 1, 2 runs)."
    )
    return dict(
        format="rustsheet-case/v1",
        description=desc,
        tags=["M9", "probe", "statistics", "f-dist", "T-2586"],
        sheets=[{"name": "Calc", "cells": cells}],
        expected={},
    )


def rows(name):
    """Section rows with Excel's cells: 'B', 'C', 'D', 'A' as oracle dicts ({'t','v'} or None)."""
    sh = F.sidecar(f"{PREFIX}-{name}")
    out = []
    for r, o in enumerate(sections(name), 1):
        o = dict(o, row=r, **{k: sh.get(f"{k}{r}") for k in "ABCD"})
        out.append(o)
    return out


if __name__ == "__main__":
    for name in ("a", "hold"):
        path = MP.CASES / f"{PREFIX}-{name}.case.json"
        case = build(name)
        print(name, len(case["sheets"][0]["cells"]), "cells", "exists" if path.exists() else "new")
        if "--write" in sys.argv and not path.exists():
            path.write_text(json.dumps(case, indent=1) + "\n")
