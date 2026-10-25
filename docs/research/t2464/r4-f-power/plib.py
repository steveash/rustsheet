"""T-2586 r4-f-power: shared helpers (imports r3-fdensity's flib / mkprobe; nothing copied)."""
from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).parent
T = HERE.parent
for p in (T / "r3-fdensity",):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import mpmath  # noqa: E402

import flib as F  # noqa: E402
import mkprobe as MK  # noqa: E402

mpf = mpmath.mpf
U = mpf(2) ** 53


def D_units(r) -> float:
    """ln(Excel) - ln(exact) in 2^-53 units."""
    return float((mpmath.log(mpf(r["excel"])) - mpmath.log(r["exact"])) * U)


def r3_rows(name):
    return F.with_exact(MK.rows(name))


def own_rows(name):
    """This directory's probes (mkprobe.py), with exact values."""
    import probe4 as mkprobe

    return F.with_exact(mkprobe.rows(name))


def load(src):
    """'r3:<name>' for r3-fdensity's probes, 'cells' for cells.tsv's F pdf cells, else this directory's probe."""
    if src == "cells":
        return F.with_exact(F.cells_rows())
    if src.startswith("r3:"):
        return r3_rows(src[3:])
    return own_rows(src)


def runs(src, minmax: float = 0.0, tag: str = ""):
    """(blk, rows, d1, d2) per run of SRC with max(d1, d2) >= minmax and `tag` in the run name."""
    g: dict = {}
    for r in load(src):
        g.setdefault(r["blk"], []).append(r)
    for blk, rs in g.items():
        m, n = rs[0]["d1"], rs[0]["d2"]
        if max(m, n) >= minmax and tag in blk:
            yield blk, rs, m, n


def run_rows(src, want):
    """The rows of one run, with its shape."""
    rs = [r for r in load(src) if r["blk"] == want]
    return rs, rs[0]["d1"], rs[0]["d2"]
