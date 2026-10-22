"""T-2122: K2 micro-ladder probe cases (HANDOFF2 section 12.1, K2 pickup step 1).

Runs of 256 consecutive doubles, B = GAMMALN.PRECISE(x):
- anchors x0 in {2.05, 2.25, 2.5, 2.75, 2.95}: a run starting every 2e-5 across 3e-3 (150 runs per anchor);
- near 2: a run starting at x = 2 + k*1e-5, k = 1..300.
Packed 50 runs (12,800 rows) per case for the anchors and 60 runs (15,360 rows) per case near 2, so every case
stays under the 16,384-row fill limit. The per-run c3 unwrap (review3/final/c3all.py) finds the runs by
consecutive-double breaks, so the packing order does not matter.
usage: python3 gen_k2micro.py OUTDIR
"""

import json
import math
import os
import sys

RUN = 256


def runs_for_anchor(x0):
    return [x0 + i * 2e-5 for i in range(150)]


def near2_runs():
    return [2 + k * 1e-5 for k in range(1, 301)]


def write_case(outdir, name, starts, desc):
    cells = {}
    r = 1
    for s in starts:
        x = s
        for _ in range(RUN):
            cells["A%d" % r] = x
            cells["B%d" % r] = "=GAMMALN.PRECISE(A%d)" % r
            x = math.nextafter(x, math.inf)
            r += 1
    case = {
        "format": "rustsheet-case/v1",
        "description": "Kernel-mystery probe (T-2122, K2 micro-ladder): %d runs of %d consecutive doubles, %s. "
        "B = GAMMALN.PRECISE(x)." % (len(starts), RUN, desc),
        "tags": ["M9", "probe", "numeric", "gamma"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(os.path.join(outdir, "kmyst-gam-%s.case.json" % name), "w") as f:
        json.dump(case, f, indent=1)
        f.write("\n")
    return name


def main():
    outdir = sys.argv[1]
    names = []
    for tag, x0 in zip("abcde", (2.05, 2.25, 2.5, 2.75, 2.95)):
        st = runs_for_anchor(x0)
        for p in range(3):
            part = st[50 * p : 50 * (p + 1)]
            names.append(
                write_case(
                    outdir,
                    "k2m%s%d" % (tag, p),
                    part,
                    "run starts %r + i*2e-5, i = %d..%d" % (x0, 50 * p, 50 * p + 49),
                )
            )
    st = near2_runs()
    for p in range(5):
        part = st[60 * p : 60 * (p + 1)]
        names.append(
            write_case(
                outdir, "k2mz%d" % p, part, "run starts 2 + k*1e-5, k = %d..%d" % (60 * p + 1, 60 * p + 60)
            )
        )
    print("\n".join("kmyst-gam-" + n for n in names))


if __name__ == "__main__":
    main()
