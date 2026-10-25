"""Probe t2513-r3-m1-residual-gl: Excel's GAMMALN at every shape argument (min, max, a + b) of probes a and hold,
the old pdf cells and probe1 with min(a, b) <= 1 (a primitive the density model takes as input; the hold rows'
densities are not read). A = the argument (echo), B = GAMMALN(A).  Usage: mkprobe_gl.py"""
import json

import m1lib as M

if __name__ == "__main__":
    args = set()
    shapes = set()
    for nm in ("a", "hold"):
        shapes |= {(a, b) for _, a, b in json.loads((M.HERE / f"probe_{nm}.meta.json").read_text())["rows"]}
    shapes |= {(r["a"], r["b"]) for r in M.load_old() + M.load_probe1()}
    for a, b in shapes:
        args |= {a, b, a + b}
    args = sorted(v for v in args if v > 0)
    cells = {}
    for i, v in enumerate(args):
        cells[f"A{i + 1}"] = "=" + M.lit(v)
        cells[f"B{i + 1}"] = f"=GAMMALN(A{i + 1})"
    case = {"format": "rustsheet-case/v1",
            "description": "T-2513 r3-m1-residual probe gl: Excel's GAMMALN at the shape arguments of the "
            "r3-m1-residual density probes (exact literals echoed in A)",
            "tags": ["M9", "probe", "statistics", "T-2513", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    (M.SUITE / "t2513-r3-m1-residual-gl.case.json").write_text(json.dumps(case, indent=1) + "\n")
    (M.HERE / "probe_gl.args.json").write_text(json.dumps(args) + "\n")
    print(len(args), "arguments")
