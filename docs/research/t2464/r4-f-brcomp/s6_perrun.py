"""Per run: r (smaller ratio), side, ln F, CR's and a ratlib chain's within-run residual sd and offset (units).
Usage: s6_perrun.py SET pw,hl,grp,est [min max(d)]  (ratlib knobs, L = xl)"""
import statistics as S
import sys

import mpmath

import fblib as B
import ratlib as RL

SET = sys.argv[1]
pw, hl, grp, est = sys.argv[2].split(",")
MIN = float(sys.argv[3]) if len(sys.argv) > 3 else 0
o = dict(pw=pw, hl=hl, grp=grp, est=est, L="xl")
rows = [r for r in B.load(SET) if max(r["d1"], r["d2"]) >= MIN]
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
print(f"# {SET} chain {sys.argv[2]}: run | r side lnF | CR off/sd | model off/sd | model exact/w4/w15")
for blk, rr in runs.items():
    r0 = rr[0]
    m, n, x = r0["d1"], r0["d2"], r0["x"]
    rat = min(m * x / n, n / (m * x))
    side = 1 if m * x <= n else 2
    lnF = float(mpmath.log(r0["exact"]))
    uc = [B.F.units(r["cr"], r["excel"]) for r in rr]
    vm = [RL.value(r["x"], m, n, o) for r in rr]
    um = [B.F.units(v, r["excel"]) for v, r in zip(vm, rr)]
    t = B.F.tally(list(zip(vm, [r["excel"] for r in rr])))
    print(f"{blk:22s} r={rat:.3f} s{side} lnF={lnF:8.2f} | CR {S.median(uc):+6.1f}/{S.pstdev(uc):5.1f} | "
          f"model {S.median(um):+6.1f}/{S.pstdev(um):5.1f} | {t[1]:2d}/{t[2]:2d}/{t[3]:2d} of {t[0]}")
