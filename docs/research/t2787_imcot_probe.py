"""T-2787: pick IMCOT inputs where the two denominator groupings print different 15-digit text.

Grouping A (T-2537): (cosh 2y + 1) - 2cos^2 x.  Grouping B (T-2044): cosh 2y - (2cos^2 x - 1).
Writes eval/suites/probes/cases/t2787-imcot-denominator-grouping.case.json (cells A1..; one input each).
Usage: python3 docs/research/t2787_imcot_probe.py
"""
import json
import math
import random

CASE = "eval/suites/probes/cases/t2787-imcot-denominator-grouping"


def texts(x, y):
    c, s2x, sh = math.cos(x), math.sin(2 * x), math.sinh(2 * y)
    ch = math.cosh(2 * y)
    out = []
    for t in (ch + 1.0 - 2.0 * c * c, ch - (2.0 * c * c - 1.0)):
        out.append(("%.15g" % (s2x / t), "%.15g" % (-sh / t)))
    return out


def pick(n=24):
    rng = random.Random(2787)
    found = []
    while len(found) < n:
        x = round(rng.uniform(-4, 4), 4)
        y = round(rng.uniform(-2, 2), 4)
        if y == 0:
            continue
        a, b = texts(x, y)
        if a != b:
            found.append((x, y))
    return found


def fmt(x, y):
    return ("%s%s%si" % (repr(x), "+" if y >= 0 else "-", repr(abs(y))))


pts = pick()
cells = {"A%d" % (i + 1): '=IMCOT("%s")' % fmt(x, y) for i, (x, y) in enumerate(pts)}
case = {
    "format": "rustsheet-case/v1",
    "description": "T-2787: IMCOT at 24 random x+iy (4 decimals) where the two denominator groupings, (cosh 2y + 1) - 2cos^2 x (T-2537) and cosh 2y - (2cos^2 x - 1) (T-2044), print different 15-digit text. Generator: docs/research/t2787_imcot_probe.py.",
    "tags": ["M9", "engineering", "complex", "probe"],
    "sheets": [{"name": "S", "cells": cells}],
    "expected": {},
}
json.dump(case, open(CASE + ".case.json", "w"), indent=2)
