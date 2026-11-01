"""T-3568 r5-bup: build probes t3568-r5-bup-tune and t3568-r5-bup-hold (probes-numeric), filled together.

Every point is a cell whose bratio call takes L140 bup+bgrat with n >= 2 bup terms in r5-bgrat-fdf1's frozen kernel
(fmodel.CFG): a0 > 15, 2 <= b0 < 40, x0 > 0.7. Columns, with (a0, b0, x0, y0) the oriented call, n = int(b0) and
c = b0 - n (c = 1, n - 1 at an integer b0):
  A  the caller's cell                     B  D = BETA.DIST(x0, a0, c, TRUE)   (bgrat's increment alone, w = 0)
  C  P = BETA.DIST(y0, c, a0, FALSE)       (Excel's density in bup's own orientation)
Regions: beta (BETA.DIST, non-integer b), bint (integer b, c = 1), f (F.DIST(f, d1, d2, TRUE), integer dfs, b0 = d2/2),
binom (BINOM.DIST(k, N, p, TRUE): the swapped call I_{1-p}(N-k, k+1)), negb (NEGBINOM.DIST(f, r, p, TRUE) = I_p(r, f+1)).
The D point's own branch (bgrat, or the a0 = 1 closed form at c = 1) is recorded in the meta, so scorers can split on
it. Meta: [[row, region, a0, b0, x0, y0, n, c, d_branch], ...].
usage: mkprobe.py [--fresh] [--write]   (--fresh: t3568-r5-bup-fresh, seed 35689, built after the freeze)"""
from __future__ import annotations

import json
import math
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r5-bgrat-fdf1"))
sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import fmodel  # noqa: E402
from common import CASES, w2p, xlk  # noqa: E402

NUM = w2p.num
SIZES = {"beta": 800, "bint": 150, "f": 300, "binom": 200, "negb": 200}


def dec(rnd: random.Random, v: float, k: int) -> float:
    return float(f"{v:.{k}g}")


def candidates(rnd: random.Random, region: str):
    """Yield (formula, a0, b0, x0, y0) candidates for a region."""
    while True:
        if region in ("beta", "bint"):
            b = float(rnd.randint(2, 39)) if region == "bint" else dec(rnd, rnd.uniform(2.0, 40.0), rnd.randint(3, 6))
            a = dec(rnd, math.exp(rnd.uniform(math.log(max(15.5, 2.4 * b)), math.log(3000.0))), rnd.randint(3, 6))
            hi = a / (a + b)
            if hi <= 0.7:
                continue
            x = dec(rnd, rnd.uniform(0.7, hi), rnd.randint(4, 15))
            yield f"=BETA.DIST({NUM(x)},{NUM(a)},{NUM(b)},TRUE)", a, b, x, 1.0 - x
        elif region == "f":
            d1, d2 = rnd.randint(31, 2000), rnd.randint(4, 79)
            f = dec(rnd, math.exp(rnd.uniform(-3, 0.5)), rnd.randint(4, 15))
            x, y = fmodel.xy(f, float(d1), float(d2))
            yield f"=F.DIST({NUM(f)},{d1},{d2},TRUE)", d1 / 2, d2 / 2, x, y
        elif region == "binom":
            k, big = rnd.randint(1, 38), rnd.randint(16, 2000)
            n = big + k
            p = dec(rnd, rnd.uniform((k + 1) / (n + 1), 0.3), rnd.randint(4, 15))
            # bratio(k+1, n-k, p, 1-p) swaps (lambda < 0) to (n-k, k+1, 1-p, p)
            yield f"=BINOM.DIST({k},{n},{NUM(p)},TRUE)", float(n - k), float(k + 1), 1.0 - p, p
        else:
            f, r = rnd.randint(1, 38), rnd.randint(16, 2000)
            p = dec(rnd, rnd.uniform(0.7, r / (r + f + 1)), rnd.randint(4, 15))
            yield f"=NEGBINOM.DIST({f},{r},{NUM(p)},TRUE)", float(r), float(f + 1), p, 1.0 - p


def build(seed: int):
    rnd = random.Random(seed)
    cells, meta = {}, []
    for region, size in SIZES.items():
        got, gen = 0, candidates(rnd, region)
        while got < size:
            batch = [next(gen) for _ in range(400)]
            # the caller's own oriented call: (a, b, x, y) of bratio before its swap
            pre = []
            for form, a0, b0, x0, y0 in batch:
                pre.append((a0, b0, x0, y0) if region != "binom" else (b0, a0, y0, x0))
            out = xlk.run(pre, fmodel.CFG)
            for (form, a0, b0, x0, y0), o in zip(batch, out):
                if not o[2].startswith("L140 bup+bgrat") or got >= size:
                    continue
                n, c = int(b0), b0 - int(b0)
                if c == 0.0:
                    n, c = n - 1, 1.0
                if n < 2:
                    continue
                dbr = xlk.run([(a0, c, x0, y0)], fmodel.CFG)[0][2]
                k = len(meta) + 1
                cells[f"A{k}"] = form
                cells[f"B{k}"] = f"=BETA.DIST({NUM(x0)},{NUM(a0)},{NUM(c)},TRUE)"
                cells[f"C{k}"] = f"=BETA.DIST({NUM(y0)},{NUM(c)},{NUM(a0)},FALSE)"
                meta.append([k, region, a0, b0, x0, y0, n, c, dbr])
                got += 1
    return cells, meta


def main() -> None:
    probes = (("t3568-r5-bup-tune", 35681), ("t3568-r5-bup-hold", 35682))
    if "--fresh" in sys.argv:  # the post-freeze sweep (built after model.py was committed), half size
        for k in SIZES:
            SIZES[k] //= 2
        probes = (("t3568-r5-bup-fresh", 35689),)
    for name, seed in probes:
        cells, meta = build(seed)
        case = {"format": "rustsheet-case/v1",
                "description": f"T-3568 r5-bup: L140 bup+bgrat cells with n >= 2 bup terms, Excel's D and density there "
                               f"(seed {seed}; docs/research/t2464/r5-bup)",
                "tags": ["M9", "probe", "statistics", "T-3568", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
        assert len(cells) <= 16384
        regions = {}
        for m in meta:
            regions[(m[1], m[8])] = regions.get((m[1], m[8]), 0) + 1
        print(name, len(meta), "points", len(cells), "cells", sorted(regions.items()))
        if "--write" in sys.argv:
            (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
            json.dump(meta, open(HERE / f"{name}.meta.json", "w"))


if __name__ == "__main__":
    main()
