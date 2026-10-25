"""Scores (bit-exact / within 4 ulp / within 15 digits) of CR, M-sd, M-sdm (r4-f-small-df sdmodel) and the frozen
M-bc, M-bcm (bcmodel) on a set, overall and by class (run kind x1 / sw, min(d) >= 16 or not).
Usage: final_score.py SET [SET ...]   (sets: ba, bhold, or any blib/clib/slib name)"""

from __future__ import annotations

import sys

import bcmodel as BC
import blib as B

SL, SD, F = B.SL, B.SD, B.F
MODELS = {
    "CR": lambda r: F.cr(r["exact"]),
    "M-sd": lambda r: SD.value(r["x"], r["d1"], r["d2"], "div"),
    "M-sdm": lambda r: SD.value(r["x"], r["d1"], r["d2"], "mulr"),
    "M-bc": lambda r: BC.value(r["x"], r["d1"], r["d2"], "div"),
    "M-bcm": lambda r: BC.value(r["x"], r["d1"], r["d2"], "mulr"),
}


def cls(r):
    big = "big" if min(r["d1"], r["d2"]) >= BC.BIG_MIN else "small"
    return f"{big}/{r['blk'][:2]}" if r.get("src") in ("a", "hold") else big


if __name__ == "__main__":
    for name in sys.argv[1:]:
        rows = B.load(name)
        print(f"== {name} ({len(rows)} cells)")
        for nm, fn in MODELS.items():
            t = SL.score_rows(rows, fn, cls)
            print(f"  {nm:6s} " + "  ".join(f"{k}: {F.fmt(v)}" for k, v in sorted(t.items())))
