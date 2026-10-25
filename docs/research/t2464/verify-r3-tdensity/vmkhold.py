"""Build the verifier's own hold-out t2513-verify-r3-tdensity-hold (probes-numeric): T.DIST(t, n, FALSE) at dfs
never probed by r3-tdensity's runs, independent random t (not consecutive doubles) in five q = t*t/n bands (series,
bd0 log form, either side of the claimed x^2 > n cut, and the far log branch).  Column A holds t as an exact
two-piece formula (hi*2^a + lo*2^b, each integer < 2^27, so no literal exceeds 15 digits); column B the density.
usage: vmkhold.py   (then fill with rustsheet-oracle, see RESULTS in the verifier's return)"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

OUT = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases/t2513-verify-r3-tdensity-hold.case.json")
DFS = (13, 15, 17, 19, 22, 26, 33, 37, 46, 50)
QS = (0.15, 0.5, 0.9, 1.1, 6.0)
PER = 20


def enc(t: float) -> str:
    m, e = math.frexp(t)  # t = m 2^e, 0.5 <= m < 1
    mi = int(m * 2**53)
    ex = e - 53
    hi, lo = mi >> 27, mi & (2**27 - 1)
    assert hi * 2.0 ** (ex + 27) + lo * 2.0**ex == t
    return f"=({hi}*2^{ex + 27}+{lo}*2^{ex})"


def main():
    rng = random.Random(25130424)
    cells, r = {}, 0
    for n in DFS:
        for q in QS:
            for _ in range(PER):
                r += 1
                t = math.sqrt(q * n) * (1 + rng.uniform(-0.02, 0.02))
                cells[f"A{r}"] = enc(t)
                cells[f"B{r}"] = f"=T.DIST(A{r},{n},FALSE)"
    case = {
        "format": "rustsheet-case/v1",
        "description": "T-2513 verify-r3-tdensity hold-out: T.DIST(t,n,FALSE) at dfs 13..50 not used by r3-tdensity, "
        "random exact-input t in five t*t/n bands (0.15, 0.5, 0.9, 1.1, 6). Column A is t, B the density.",
        "tags": ["M9", "probe", "statistics", "student-t", "T-2513"],
        "sheets": [{"name": "Calc", "cells": cells}],
        "expected": {},
    }
    OUT.write_text(json.dumps(case, indent=1) + "\n")
    print(f"{OUT}: {len(cells)} cells")


if __name__ == "__main__":
    main()
