"""T-2586 r5-f-smallr probes: F.DIST(x, d1, d2, FALSE) on runs of consecutive doubles with r <= 0.375 on both sides
(r = the smaller of m x/n, n/(m x)).  A = x (w2-forensics' exact encoder, echoed), B = F.DIST(A, d1, d2, FALSE).
  a     in-sample: 16 shapes x 4 runs (side 1 r ~ 0.29, 0.06; side 2 r ~ 0.31, 0.045) x 112 consecutive doubles
  hold  hold-out (built and filled with a, read once after the model is frozen): 8 other shapes x 2 runs
        (side 1 and side 2, other r) x 96
Usage: mkprobe5.py [--write]"""
from __future__ import annotations

import json
import sys

import types

import r5lib as R
import probe4 as P4  # noqa: E402  (r4-f-power, on r5lib's path)

F = R.F
MP = F.MP
PREFIX = "t2586-r5-f-smallr"
SPEC = {
    "a": ([(40, 40), (80, 80), (120, 120), (200, 200), (30, 90), (90, 30), (50, 150), (150, 50), (24, 120),
           (120, 24), (70, 110), (110, 70), (256, 64), (64, 256), (36, 36), (180, 60)],
          ((1, 0.29), (1, 0.06), (2, 0.31), (2, 0.045)), 112),
    "hold": ([(44, 100), (100, 44), (66, 66), (140, 140), (28, 56), (56, 28), (170, 90), (90, 170)],
             ((1, 0.17), (2, 0.21)), 96),
}


def runs(name):
    shapes, rt, cnt = SPEC[name]
    out = []
    i = 0
    for d1, d2 in shapes:
        for side, r in rt:
            rr = r * (1 + 0.0137 * ((7 * i) % 11) + 0.00291)
            x0 = rr * d2 / d1 if side == 1 else d2 / (d1 * rr)
            i += 1
            out += [(f"run({d1},{d2},s{side},r{r:g})", float(d1), float(d2), x) for x in MP.run(x0, 0, cnt)]
    return out


def _rebind(fn):
    """r4-f-power probe4's builder / reader, run against this module's runs() and PREFIX (reused, not copied)."""
    return types.FunctionType(fn.__code__, globals(), fn.__name__)


build = _rebind(P4.build)  # note: its description text names r4-f-power; the written case was built by an earlier,
rows = _rebind(P4.rows)    # equivalent version of this file with this directory's description


if __name__ == "__main__":
    tot = 0
    for name in SPEC:
        c = build(name)
        k = len(c["sheets"][0]["cells"])
        tot += k
        p = MP.CASES / f"{PREFIX}-{name}.case.json"
        print(name, k, "cells", "exists" if p.exists() else "new", p)
        if "--write" in sys.argv and not p.exists():  # (both cases exist; kept for the record)
            p.write_text(json.dumps(c, indent=1) + "\n")
    print("total", tot)
