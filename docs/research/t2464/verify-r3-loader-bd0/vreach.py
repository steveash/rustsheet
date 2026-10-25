"""Reachability: share of Excel outputs per block that are EXP_V6 of no double E (scan of +-64 doubles around
ln y).  A stored-E chain cannot produce such outputs.  Usage: vreach.py CASE_SUFFIX"""
from __future__ import annotations

import math
import sys

import mpmath

import vdata as V
import vmodel as M


def reachable(y: float, w: int = 64) -> bool:
    e = float(mpmath.log(y))
    lo = e
    for _ in range(w):
        lo = math.nextafter(lo, -math.inf)
    c = lo
    for _ in range(2 * w + 1):
        if M.exp_v6(c) == y:
            return True
        c = math.nextafter(c, math.inf)
    return False


def main(suf):
    rows = V.load_case(f"t2513-{suf}" if suf.startswith("verify") else f"t2513-r3-loader-bd0-{suf}")
    for blk, idx in V.blocks(rows).items():
        r0 = rows[idx[0]]
        un = sum(not reachable(rows[i]["excel"]) for i in idx)
        print(f"{blk:8s} ({r0['a']:g},{r0['b']:g}) ln y {math.log(r0['excel']):+8.3f}  unreachable {un}/{len(idx)}")


if __name__ == "__main__":
    main(sys.argv[1])
