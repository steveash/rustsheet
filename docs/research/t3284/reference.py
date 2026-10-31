"""T-3289 references: Excel's deep upper tail of Q(a, x) against 60-digit mpmath.

Reads probes-numeric/t3284-gamma-deep-upper-tail (case and oracle sidecar), evaluates each cell's
regularized upper incomplete gamma Q(a, x) = gammainc(a, x, inf, regularized=True) at the exact
double arguments Excel sees (POISSON(k, m, TRUE) = Q(k + 1, m); CHISQ.DIST.RT(x, df) =
Q(df / 2, x / 2), statistics.md "One incomplete-gamma kernel"), and prints per cell Excel's value
and its error in ulps of Excel's value, then the count within 1 ulp by family and shape.

Run: python3 -I docs/research/t3284/reference.py  (needs mpmath; e.g. `uv run --with mpmath`)
"""

import json
import math
import re
from collections import defaultdict
from pathlib import Path

import mpmath as mp

mp.mp.dps = 60
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
STEM = "t3284-gamma-deep-upper-tail"
case = json.loads((CASES / f"{STEM}.case.json").read_text())
excel = json.loads((CASES / f"{STEM}.oracle.json").read_text())["sheets"]["Calc"]
cells = case["sheets"][0]["cells"]

ARG = r"(\d+)(?:\+\((-?\d+)\)\*2\^\((-\d+)\))?"
POISSON = re.compile(rf"=POISSON\((\d+),{ARG},TRUE\)")
CHISQ = re.compile(rf"=CHISQ\.DIST\.RT\({ARG},(\d+)\)")


def exact(base: str, j: str | None, e: str | None) -> mp.mpf:
    """The double Excel evaluates `base + j * 2^e` to (exact: every grid point is representable)."""
    value = mp.mpf(base)
    if j is not None and e is not None:
        value += mp.mpf(int(j)) * mp.mpf(2) ** int(e)
    return value


def shape_and_x(formula: str) -> tuple[mp.mpf, mp.mpf, str]:
    if m := POISSON.fullmatch(formula):
        a = mp.mpf(int(m.group(1)) + 1)
        family = "poisson-ulp" if m.group(3) else "poisson"
        return a, exact(m.group(2), m.group(3), m.group(4)), family
    m = CHISQ.fullmatch(formula)
    if m is None:
        raise ValueError(formula)
    df = int(m.group(4))
    family = "chisq-ulp" if m.group(2) else ("chisq-twin" if df % 2 == 0 else "chisq-half")
    return mp.mpf(df) / 2, exact(m.group(1), m.group(2), m.group(3)) / 2, family


within: dict[str, list[int]] = defaultdict(lambda: [0, 0])
for key in sorted(cells, key=lambda k: int(k[1:])):
    a, x, family = shape_and_x(cells[key])
    got = excel[key]["v"]
    ref = mp.gammainc(a, x, mp.inf, regularized=True)
    err = float((mp.mpf(got) - ref) / mp.mpf(math.ulp(got)))
    print(f"{key:5} {family:11} a={mp.nstr(a, 6):5} x/a={mp.nstr(x / a, 6):10} {got!r:26} {err:+8.1f}")
    for group in (family, f"{family} a={mp.nstr(a, 4)}", "all"):
        within[group][0] += abs(err) <= 1
        within[group][1] += 1
print()
for group, (ok, n) in sorted(within.items()):
    print(f"within 1 ulp: {group:24} {ok:3} / {n}")
