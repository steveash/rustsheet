"""T-3289: count the cells of t3284-gamma-deep-upper-tail where Excel is within 1 ulp of the 50-digit value.

Usage: python3 ulp_vs_exact.py  (needs mpmath; reads the case and its oracle sidecar from the repo)
"""
import json
import math
import re
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
CASES = Path(__file__).resolve().parents[3] / "eval/suites/probes-numeric/cases"
STEM = "t3284-gamma-deep-upper-tail"


def arg(s):
    """Evaluate a literal or `a+(j)*2^(-e)` argument exactly."""
    return mp.mpf(eval(s.replace("^", "**"), {}, {}))  # noqa: S307 - own fixture text


def exact(formula):
    m = re.fullmatch(r"=POISSON\((\d+),(.+),TRUE\)", formula)
    if m:
        return mp.gammainc(int(m[1]) + 1, arg(m[2]), mp.inf, regularized=True)
    m = re.fullmatch(r"=CHISQ\.DIST\.RT\((.+),(\d+)\)", formula)
    return mp.gammainc(mp.mpf(int(m[2])) / 2, arg(m[1]) / 2, mp.inf, regularized=True)


cells = json.loads((CASES / f"{STEM}.case.json").read_text())["sheets"][0]["cells"]
oracle = json.loads((CASES / f"{STEM}.oracle.json").read_text())["sheets"]["Calc"]
within = 0
for ref, formula in cells.items():
    xl = oracle[ref]["v"]
    ulp = math.ulp(xl)
    err = float((mp.mpf(xl) - exact(formula)) / ulp)
    within += abs(err) <= 1
    print(f"{ref}\t{formula}\t{err:+.1f}")
print(f"within 1 ulp: {within} of {len(cells)}")
