"""b8 (exploratory, after the freeze): M-bc's switch threshold BIG_MIN (min(d) at which the e-table part is dropped)
on sets with mid shapes. Usage: b8_thresh.py SET [SET ...]"""

from __future__ import annotations

import sys

import bcmodel as BC
import blib as B

if __name__ == "__main__":
    for name in sys.argv[1:]:
        rows = [r for r in B.load(name) if 6 <= min(r["d1"], r["d2"]) < 16]
        for t in (6, 8, 10, 12, 14, 16):
            BC.BIG_MIN = t
            BC.K_of.cache_clear()
            res = B.SL.score_rows(rows, lambda r: BC.value(r["x"], r["d1"], r["d2"], "div"))
            print(name, "min(d) in [6,16): BIG_MIN", t, B.F.fmt(res["all"]))
