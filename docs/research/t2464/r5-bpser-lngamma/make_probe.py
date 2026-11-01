"""T-3570 probes t3570-r5-bpser-lngamma-{beta,gl}: separate the two lnGamma(a0 < 1) forms inside bpser's L.

beta: BETA.DIST(x, a0, b0, TRUE) at fixed b0 in {2.5, 6.25} (TOMS sub-branch 1 < b0 < 8), with a0 stepping by single
      ulps (160 consecutive doubles) around 0.3, 0.5 and 0.7, at two short-decimal x per run where bpser fires
      (w_bpser, direct orientation). Exact doubles are written with w2-forensics' `num` (no 15-digit truncation).
gl:   =GAMMALN(v) at every a0 and fl(a0 + b0) of beta, so L uses Excel's measured GAMMALN (r2-bpser-small item 2).
usage: make_probe.py [--write]
"""

from __future__ import annotations

import importlib.util
import json
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("bpser_small_lib", HERE.parent / "r2-bpser-small" / "lib.py")
lib = importlib.util.module_from_spec(_spec)
sys.modules["bpser_small_lib"] = lib
_spec.loader.exec_module(lib)

B0S = (2.5, 6.25)
CENTERS = (0.3, 0.5, 0.7)
HALF = 80  # ulps below and above each center
XS = {  # two short-decimal x per b0, chosen once (bpser_fires checked in build)
    2.5: (0.0371, 0.0913),
    6.25: (0.0213, 0.0371),
}


def a0_run(c: float) -> list[float]:
    v = c
    for _ in range(HALF):
        v = math.nextafter(v, 0.0)
    out = []
    for _ in range(2 * HALF):
        out.append(v)
        v = math.nextafter(v, 1.0)
    return out


def build() -> dict[str, list]:
    beta, i = [], 0
    for b0 in B0S:
        for c in CENTERS:
            for x in XS[b0]:
                for a0 in a0_run(c):
                    assert lib.kernel(a0, b0, x, 1.0 - x)[2].split("/")[0] == "w_bpser", (a0, b0, x)
                    i += 1
                    beta.append((i, f"{b0}:{c}:{x}", x, a0, b0))
    vals = sorted({v for _, _, _, a, b in beta for v in (a, a + b)})
    return {"beta": beta, "gl": [(k, "gl", v, v, v) for k, v in enumerate(vals, 1)]}


def main() -> None:
    out = build()
    for nm, rows in out.items():
        fn = "=GAMMALN({0})" if nm == "gl" else "=BETA.DIST({0},{1},{2},TRUE)"
        cells = {f"A{i}": fn.format(lib.w2p.num(x), lib.w2p.num(a), lib.w2p.num(b)) for i, _, x, a, b in rows}
        name = f"t3570-r5-bpser-lngamma-{nm}"
        print(name, len(cells), "cells")
        if "--write" in sys.argv:
            case = {
                "format": "rustsheet-case/v1",
                "description": f"T-3570 r5-bpser-lngamma: lnGamma(a0 < 1) inside bpser's L, a0 by single ulps ({nm})",
                "tags": ["M9", "probe", "statistics", "T-2464", "T-3570", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}],
                "expected": {},
            }
            (lib.CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
            json.dump(rows, open(HERE / f"{name}.meta.json", "w"))


if __name__ == "__main__":
    main()
