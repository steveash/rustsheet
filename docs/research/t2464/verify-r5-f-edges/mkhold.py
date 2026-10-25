"""verify-r5-f-edges: builds the verifier's own hold-out t2586-verify-r5-f-edges-hold (<= 2,000 cells) at
shapes no earlier F probe or cells.tsv row used (checked at build time). Columns: A = exact x formula,
B = F.DIST(A,d1,d2,FALSE), C = B*2^600 where the density can be below 1E-200, D = truncated-df twin."""
from __future__ import annotations

import glob
import json
import math
import re
import sys
from fractions import Fraction
from pathlib import Path

import mpmath as mp

import vparse

mp.mp.prec = 200
HERE = Path(__file__).resolve().parent
CASE = "t2586-verify-r5-f-edges-hold"


def used_shapes() -> set:
    used = set()
    pat = re.compile(r"F\.DIST(?:\.RT)?\(([^,]*),([^,]*),([^,]*)[,)]", re.I)
    for f in glob.glob(str(vparse.CASES / "*.case.json")):
        if CASE in f:
            continue
        try:
            c = json.loads(Path(f).read_text())
        except (OSError, ValueError):
            continue
        for s in c.get("sheets", []):
            for v in s.get("cells", {}).values():
                if isinstance(v, str):
                    for m in pat.finditer(v):
                        try:
                            used.add((math.trunc(vparse.ev(m.group(2))), math.trunc(vparse.ev(m.group(3)))))
                        except Exception:  # noqa: BLE001 - non-literal df, not a shape
                            pass
    for line in (HERE.parent / "cells.tsv").read_text().splitlines()[1:]:
        p = line.split("\t")
        if p[0] in ("F.DIST", "FDIST", "F.DIST.RT"):
            a = p[3].split("|")
            try:
                used.add((math.trunc(float(a[1])), math.trunc(float(a[2]))))
            except ValueError:
                pass
    return used


def enc(x: float) -> str:
    """Exact Excel formula for a positive double: (H*2^26+L)*2^E (split 2^-500 for small E)."""
    if x == 0:
        return "=0"
    s = "-" if x < 0 else ""
    m, e = math.frexp(abs(x))
    M = int(m * 2**53)
    E = e - 53
    assert Fraction(M) * Fraction(2) ** E == Fraction(abs(x))
    H, L = divmod(M, 2**26)
    core = f"({H}*2^26+{L})"
    if E < -1000:
        body = f"{core}*2^-500*2^{E + 500}"
    else:
        body = f"{core}*2^{E}"
    return f"={s}({body})" if s else f"={body}"


def logpdf(x, m, n):
    X, a, b = mp.mpf(x), mp.mpf(m) / 2, mp.mpf(n) / 2
    return (mp.loggamma(a + b) - mp.loggamma(a) - mp.loggamma(b) + a * mp.log(mp.mpf(m) / n)
            + (a - 1) * mp.log(X) - (a + b) * mp.log1p(mp.mpf(m) * X / n))


def solve(target_ln, m, n, lo, hi):
    """x in [lo, hi] (log-bisection) with ln(target(x)) = target_ln, target = pdf*x (fx) given by fn."""
    f = lambda lx: target_ln[0](mp.e ** lx, m, n) - target_ln[1]  # noqa: E731
    a, b = mp.log(lo), mp.log(hi)
    fa = f(a)
    for _ in range(200):
        c = (a + b) / 2
        fc = f(c)
        if (fc > 0) == (fa > 0):
            a, fa = c, fc
        else:
            b = c
    return float(mp.e ** a)


def x87_mul_overflows(m: float, x: float) -> bool:
    """m*x rounded to 64 bits (unbounded exponent), then to a double: overflow?"""
    p = Fraction(m) * Fraction(x)

    def rnd(q: Fraction, bits: int) -> Fraction:
        e = q.numerator.bit_length() - q.denominator.bit_length()
        while Fraction(2) ** e > q:
            e -= 1
        while Fraction(2) ** (e + 1) <= q:
            e += 1
        sc = Fraction(2) ** (bits - 1 - e)
        v = q * sc
        f = math.floor(v)
        r = v - f
        if r > Fraction(1, 2) or (r == Fraction(1, 2) and f % 2):
            f += 1
        return Fraction(f) / sc

    return rnd(rnd(p, 64), 53) >= Fraction(2) ** 1024


def build():
    used = used_shapes()
    rows = []  # (kind, x formula, d1 text, d2 text, C?, twin (d1,d2) or None)

    def add(kind, x, d1, d2, c=False, twin=None, xf=None):
        rows.append((kind, xf or enc(x), d1, d2, c, twin))

    def fresh(*shapes):
        for s in shapes:
            assert s not in used, f"shape {s} already used"

    # 1. rules
    for d1 in ("1.1", "1.9999999", "2.0000001", "2.5", "2.9999999", "3", "3.9", "7", "10000000000.5"):
        for d2 in ("1.3", "160", "10000000000"):
            add("zero", 0.0, d1, d2, xf="=0")
    for xf in ("=-5", "=-(2^-1022)", "=-1E-300"):
        for d1, d2 in (("1", "124"), ("2", "500"), ("9", "130")):
            add("neg", 0, d1, d2, xf=xf)
    for d in ("0.3", "0.999999999999999", "-0.5", "10000000001", "100000000000"):
        add("df", 1.5, d, "124")
        add("df", 1.5, "124", d)
    for d in ("10000000000.99", "1.000000001", "2.999999999"):
        add("dfok", 1.5, d, "124", twin=(str(math.trunc(float(d))), "124"))
        add("dfok", 1.5, "124", d, twin=("124", str(math.trunc(float(d)))))
    for d1 in ("1", "2", "5", "9", "130"):
        for x in (2.0**-1030, 3.3 * 2.0**-1060):
            add("subx", x, d1, "17")
        add("subx", 0, d1, "17", xf="=EXP(-725)")
        add("minnorm", 2.0**-1022, d1, "17", c=True)
    # 2. overflow edges (fl(trunc(d1)*x))
    for m in (4, 9, 11, 130, 1000):
        x = sys.float_info.max / m
        while x87_mul_overflows(m, x):
            x = math.nextafter(x, 0)
        while not x87_mul_overflows(m, math.nextafter(x, math.inf)):
            x = math.nextafter(x, math.inf)
        for k in (-1, 0, 1, 2):
            xx = x
            for _ in range(abs(k)):
                xx = math.nextafter(xx, math.inf if k > 0 else 0)
            add("ovf", xx, str(m), "3", c=True)
        if m == 4:
            add("ovftr", x, "4.9", "3", c=True, twin=("4", "3"))  # fl(5x) overflows, fl(4x) not
            add("ovftr", x, "4.0000001", "1", c=True, twin=("4", "1"))
    # 3. twins at new truncated shapes
    tw = [("1.6", "160.3"), ("2.9", "500.5"), ("130.7", "1.2"), ("500.01", "2.99"), ("11.5", "13.5")]
    fresh(*[(math.trunc(float(a)), math.trunc(float(b))) for a, b in tw])
    for a, b in tw:
        m, n = math.trunc(float(a)), math.trunc(float(b))
        for t in (0.03, 0.2, 0.6, 1.0, 1.7, 3.1, 7.0, 25.0):
            x = float(Fraction(t) * n / m)
            add("twin", x, a, b, twin=(str(m), str(n)))
    # 4. runs of consecutive doubles at d = 1, 2 with new partners
    vs = [(1, 124), (1, 500), (1, 1000), (2, 150), (2, 500), (2, 1000),
          (124, 1), (500, 1), (1000, 1), (130, 2), (500, 2), (1000, 2)]
    fresh(*vs)
    for m, n in vs:
        for t in (0.11, 4.3):
            x = float(Fraction(t) * n / m)
            for _ in range(12):
                add("vrun", x, str(m), str(n))
                x = math.nextafter(x, math.inf)
    # 5. tiny / huge x at the same shapes
    for m, n in vs:
        for k in (-280, -150, -60, -12, 12, 60, 150, 280):
            x = float(mp.mpf(10) ** k)
            add("tail", x, str(m), str(n), c=True)
    # 6. thresh: Y = F*x crossing the subnormal range, side 1, a >= 1.5
    th = [(3, 1000), (5, 130), (9, 130), (17, 5)]
    fresh(*th)
    fx = lambda x, m, n: logpdf(x, m, n) + mp.log(x)  # noqa: E731
    for m, n in th:
        for ly in (-1074.5, -1073.6, -1073, -1072, -1070, -1067, -1063, -1058, -1050, -1040, -1030, -1020,
                   -1010, -995, -950):
            x = solve((fx, ly * mp.log(2)), m, n, 1e-320 if m > 50 else 1e-300, 1e-3)
            add("thresh", x, str(m), str(n), c=True)
    # 7. hthr: F crossing 2^-1022 at huge x (side 2)
    hs = [(17, 5), (11, 13)]
    fresh(*hs)
    for m, n in hs:
        for lf in (-1000, -1015, -1020, -1021.5, -1022, -1022.3, -1023, -1025, -1030, -1040, -1060, -1070):
            x = solve((logpdf, lf * mp.log(2)), m, n, 10.0, 1e300)
            add("hthr", x, str(m), str(n), c=True)
    return rows


def write():
    rows = build()
    cells, kinds = {}, {}
    for i, (kind, xf, d1, d2, c, twin) in enumerate(rows, 1):
        cells[f"A{i}"] = xf
        cells[f"B{i}"] = f"=F.DIST(A{i},{d1},{d2},FALSE)"
        if c:
            cells[f"C{i}"] = f"=F.DIST(A{i},{d1},{d2},FALSE)*2^600"
        if twin:
            cells[f"D{i}"] = f"=F.DIST(A{i},{twin[0]},{twin[1]},FALSE)"
        kinds[i] = kind
    assert len(cells) <= 2000, len(cells)
    case = {"format": "rustsheet-case/v1",
            "description": "T-2586 verify-r5-f-edges: verifier's hold-out for F.DIST(x,d1,d2,FALSE) edges at new shapes "
                           "(rules, d1*x overflow, df twins, d = 1, 2 runs with partners 124-1000, tails, underflow).",
            "tags": ["M9", "probe", "statistics", "f-dist", "T-2586"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    (vparse.CASES / f"{CASE}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (HERE / "hold_kinds.json").write_text(json.dumps(kinds))
    print(len(rows), "rows", len(cells), "cells")


if __name__ == "__main__":
    write()
