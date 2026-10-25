"""Oracle probe t2464-r2-bpser-small-tiny2: the tiny (apser / fpser) rows of probe -tiny again, with every argument
written as exact arithmetic of exact pieces (w2p.enc), plus the parse error of each tiny literal,
=(literal) - (exact pieces) / (exact pieces), to test whether Excel's parser misreads 3-digit literals like 1.87e-17.
usage: make_probe_tiny2.py [--write]"""
import json
import sys

import lib

NAME = "t2464-r2-bpser-small-tiny2"
rows = json.load(open(lib.HERE / "t2464-r2-bpser-small-tiny.meta.json"))
cells, meta = {}, []
enc = lib.w2p.enc
for i, blk, x, a, b in rows:
    cells[f"A{i}"] = f"=BETA.DIST({enc(x)},{enc(a)},{enc(b)},TRUE)"
    meta.append((i, blk, x, a, b))
tinies = sorted({b for _, _, _, _, b in rows})
for j, b in enumerate(tinies, 1):
    cells[f"B{j}"] = f"=({b!r}-{enc(b)})/{enc(b)}"
    meta.append((f"B{j}", "parse", b, b, b))
print(len(cells), "cells")
if "--write" in sys.argv:
    case = {"format": "rustsheet-case/v1",
            "description": "T-2464 r2-bpser-small: apser/fpser cells with exact arguments, and tiny-literal parse errors",
            "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    (lib.CASES / f"{NAME}.case.json").write_text(json.dumps(case, indent=1) + "\n")
    json.dump(meta, open(lib.HERE / f"{NAME}.meta.json", "w"))
