"""T-2512 r3-bgrat: shared paths, probe readers and the score table (one definition for the scripts here)."""
from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
CASES = HERE.parents[3] / "eval" / "suites" / "probes-numeric" / "cases"
sys.path.insert(0, str(T2464 / "r2-tdist"))
import xlk  # noqa: E402,F401  (r2-tdist's kernel; also puts r2-bratio on the path)
import score_lib as sl  # noqa: E402

_spec = importlib.util.spec_from_file_location("w2probes", T2464 / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(w2p)
SPLIT = "t2512-r3-bgrat-split"


def sheet(name: str) -> dict:
    """The Calc sheet of a probe's oracle sidecar."""
    return json.load(open(CASES / f"{name}.oracle.json"))["sheets"]["Calc"]


def meta(name: str) -> list:
    return json.load(open(HERE / f"{name}.meta.json"))


def num(c) -> float | None:
    return c["v"] if c and c.get("t") == "n" else None


def split_cells():
    """Probe -split rows with a number: (a, b0, x, case, Excel's D)."""
    out = sheet(SPLIT)
    rows = []
    for k, a, b0, x, case, _cell in meta(SPLIT):
        d = num(out.get(f"A{k}"))
        if d is not None:
            rows.append((a, b0, x, case, d))
    return rows


def write_probe(name: str, desc: str, cells: dict, rows: list, write: bool, extra_tags: tuple = ()) -> None:
    """Print the probe's size; with write, store its case file and the meta rows."""
    case = {"format": "rustsheet-case/v1", "description": desc,
            "tags": ["M9", "probe", "statistics", "T-2512", "incomplete-beta", *extra_tags],
            "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
    assert len(cells) <= 16384
    p = CASES / f"{name}.case.json"
    print(len(rows), "points", len(cells), "cells", p)
    if write:
        p.write_text(json.dumps(case, indent=1) + "\n")
        json.dump(rows, open(HERE / f"{name}.meta.json", "w"))


def table(g: dict, head: tuple[str, ...]) -> None:
    """Print exact/w4/w15 per key of g ({key tuple: [(model, excel)]}); head names the key columns."""
    print("| " + " | ".join(head) + " | n | exact/w4/w15 |")
    print("|" + "---|" * (len(head) + 2))
    for k in sorted(g, key=str):
        n, e, f, w = sl.tally(g[k])
        print("| " + " | ".join(str(p) for p in k) + f" | {n} | {e}/{f}/{w} |")


def split_kernel():
    """r2-tdist FINAL's kernel with its caches filled for every -split point, and the -split rows."""
    import branches

    rows = split_cells()
    xlk.run([(a, b0, x, 1.0 - x) for a, b0, x, _c, _d in rows], branches.FINAL)
    return xlk.bx._KERNELS[branches.FINAL], rows


def split_d() -> dict:
    """{(a, b0, x): Excel's D} from probe -split."""
    return {(a, b0, x): d for a, b0, x, _c, d in split_cells()}


def bupdens_p() -> dict:
    """{(a, b0, x): (BETA.DIST(x, a, b0, FALSE), BETA.DIST(y, b0, a, FALSE))} from probe -bupdens."""
    out = sheet("t2512-r3-bgrat-bupdens")
    return {(a, b0, x): (num(out.get(f"A{k}")), num(out.get(f"B{k}"))) for k, a, b0, x in meta("t2512-r3-bgrat-bupdens")}
