"""Step analysis: change of ln(Excel) between consecutive x doubles in a run (units 2^-53), and the exact change.
Excel's own value moving in multiples of 2^k units shows which stored terms carry the x dependence.
Usage: s7_steps.py SET [run substring ...]"""
import sys

import mpmath

import fblib as B

U = mpmath.mpf(2) ** 53
SET = sys.argv[1]
pats = sys.argv[2:]
rows = B.load(SET)
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
for blk, rr in runs.items():
    if pats and not any(p in blk for p in pats):
        continue
    dx = [float((mpmath.log(b["excel"]) - mpmath.log(a["excel"])) * U) for a, b in zip(rr, rr[1:])]
    de = [float((mpmath.log(b["exact"]) - mpmath.log(a["exact"])) * U) for a, b in zip(rr, rr[1:])]
    print(f"{blk}: exact step mean {sum(de) / len(de):+.1f}")
    print("   excel steps:", " ".join(f"{v:.0f}" for v in dx))
