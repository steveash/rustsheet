"""Oracle probes t2464-r2-bpser-small-{shapes,runs,tiny}: BETA.DIST(x, a, b, TRUE) on MANY distinct shapes in the
bpser region min(a, b) <= 1, few x per shape, so a fit cannot hide in a few shapes; plus consecutive-double x runs per
TOMS bpser sub-branch, and the apser / fpser branches (a or b < 1e-15).

shapes: ~1,400 shapes (p, q), p in [0.05, 1) (irregular golden-ratio steps, 4 decimals), q log-uniform in [0.3, 30]
        (4 significant digits), orientation (a, b) = (p, q) or (q, p) alternating; per shape 6 x chosen among
        short decimals so that the BEST dispatch fires a bpser call (w_bpser or w1_bpser), spread over [x_lo, x_hi].
runs:   16 runs of 512 consecutive doubles, irregular starts, one or more per sub-branch.
gl:     =GAMMALN(v) at every shape's a, b and a + b.
tiny:   apser (a0 < 1e-15 after the swap) and fpser (b0 < 1e-15) shapes, 8 x each.
usage: make_probe.py [--write] [only=NAME]
"""

from __future__ import annotations

import json
import math
import sys

import lib

PHI = (math.sqrt(5) - 1) / 2


def shape_list(n):
    out = []
    for k in range(n):
        p = round(0.05 + 0.95 * ((k * PHI + 0.0314159) % 1.0), 4)
        u = (k * 0.7548776662 + 0.271828) % 1.0
        q = float(f"{0.3 * 100.0**u:.4g}")
        if p == 0 or p == 1.0 or q == 1.0 or p == q:
            continue
        out.append((p, q) if k % 2 == 0 else (q, p))
    return out


def bpser_fires(a, b, x):
    r = lib.kernel(a, b, x, 1.0 - x)
    return any(t[0] == "bpser" for t in r[3]) and r[2].split("/")[0] in ("w_bpser", "w1_bpser")


def xs_for(a, b, k, nx=6):
    """nx short-decimal x in (0, 1) where bpser fires, spread by an irregular quasi-random sequence."""
    got = []
    for j in range(400):
        u = (k * 0.381966 + j * PHI + 0.1234) % 1.0
        # log-spread over [1e-4, 0.9999] to cover small x (where x^a matters) and large x
        x = float(f"{10 ** (-4 * (1 - u)) if j % 2 else u:.5g}")
        if not 0 < x < 1 or x in got:
            continue
        if bpser_fires(a, b, x):
            got.append(x)
        if len(got) == nx:
            break
    return got


RUNS = [  # (a, b, x0, label) chosen per sub-branch (comments: bpser's own (a, b) and sub-branch)
    (0.37, 0.81, 0.2137, "b0<=1 a<b"), (0.83, 0.29, 0.3419, "b0<=1 a>b"),
    (0.153, 0.571, 0.0731, "b0<=1 small a"), (0.43, 3.7, 0.01237, "1<b0<8 (a small)"),
    (2.9, 0.61, 0.3117, "1<b0<8 (b small)"), (0.717, 6.31, 0.0213, "1<b0<8 b near 8"),
    (5.13, 0.217, 0.4113, "1<b0<8 (b tiny-ish)"), (0.29, 13.7, 0.00313, "b0>=8 (a small)"),
    (17.3, 0.47, 0.2113, "b0>=8 (b small)"), (0.61, 4.3, 0.4171, "w1_bpser 1<b0<8"),
    (0.37, 21.3, 0.3511, "w1_bpser b0>=8"), (0.91, 2.17, 0.06171, "1<b0<8 a near 1"),
    (1.37, 0.73, 0.4531, "1<max<8 a>1"), (0.0731, 0.913, 0.1717, "b0<=1 tiny a"),
    (11.9, 0.83, 0.3713, "b0>=8 b near 1"), (0.53, 1.13, 0.05713, "1<b0<8 b near 1"),
]


def build():
    lib.bx.fetch_missing()
    out = {}
    # shapes
    rows, i = [], 0
    for k, (a, b) in enumerate(shape_list(2000)):
        for x in xs_for(a, b, k):
            i += 1
            rows.append((i, "shape", x, a, b))
        if i > 16384 - 8:
            break
    out["shapes"] = rows
    # runs
    rows, i = [], 0
    for a, b, x0, lab in RUNS:
        for x in w2p_run(x0, 512):
            i += 1
            rows.append((i, f"run:{a},{b},{x0}", x, a, b))
    out["runs"] = rows
    # tiny: apser (a < 1e-15 after the swap: BETA.DIST(x > 0.5, a, b tiny) -> a0 = b) and fpser (b0 tiny)
    rows, i = [], 0
    for k in range(48):
        big = float(f"{0.3 * 50.0 ** ((k * PHI) % 1.0):.4g}")
        tiny = float(f"{10 ** (-15.2 - 3 * ((k * 0.7548776662) % 1.0)):.3g}")
        for j in range(8):
            u = (k * 0.381966 + j * PHI) % 1.0
            if k % 2 == 0:  # fpser: direct, b tiny, x <= 0.5
                a, b, x = big, tiny, float(f"{0.5 * 10 ** (-3 * u):.4g}")
            else:  # apser: swapped (x > 0.5), b tiny -> a0 = b, b0 = a; needs b0*x0 <= 1 with x0 = 1-x
                a, b = big, tiny
                x = float(f"{1 - min(0.5, 1.0 / big) * 10 ** (-2 * u):.4g}")
                if not 0.5 < x < 1:
                    continue
            i += 1
            rows.append((i, "fpser" if k % 2 == 0 else "apser", x, a, b))
    out["tiny"] = rows
    # gl: Excel's GAMMALN at every shape's a, b and a + b (x87 sum = double sum), to separate lnB from the series
    vals = sorted({v for nm in ("shapes", "runs") for _, _, _, a, b in out[nm] for v in (a, b, a + b)})
    out["gl"] = [(i, "gl", v, v, v) for i, v in enumerate(vals, 1)]
    return out


def w2p_run(x0, n):
    return lib.w2p.run(x0, 0, n)


def main():
    out = build()
    only = [a.split("=", 1)[1] for a in sys.argv if a.startswith("only=")]
    for nm, rows in out.items():
        assert len(rows) <= 16384
        if only and nm not in only:
            continue
        fn = "=GAMMALN({0})" if nm == "gl" else "=BETA.DIST({0},{1},{2},TRUE)"
        cells = {f"A{i}": fn.format(lib.w2p.num(x), lib.w2p.num(a), lib.w2p.num(b)) for i, _, x, a, b in rows}
        name = f"t2464-r2-bpser-small-{nm}"
        shapes = len({(a, b) for _, _, _, a, b in rows})
        print(name, len(cells), "cells", shapes, "shapes")
        if "--write" in sys.argv:
            case = {
                "format": "rustsheet-case/v1",
                "description": f"T-2464 r2-bpser-small: BETA.DIST cdf, bpser region min(a,b) <= 1 ({nm})",
                "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}],
                "expected": {},
            }
            (lib.CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
            json.dump(rows, open(lib.HERE / f"{name}.meta.json", "w"))


if __name__ == "__main__":
    main()
