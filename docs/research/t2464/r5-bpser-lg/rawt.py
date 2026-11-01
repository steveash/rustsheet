"""T-3570 r5-bpser-lg: does bpser call Excel's GAMMALN kernel on [1, 2) with t = a0 itself instead of t = fl(fl(1 + a0) - 1)?
Excel's GAMMALN(x), x = fl(1 + a0) (statistics.md "GAMMALN is Excel's own x87 routine"): K3 = fl(t1 R) on [1, 1.5] and
fl(t1 fl(t2 R2)) on (1.5, 2], t1 = fl(x - 1), t2 = fl(x - 2), R and R2 Excel's stored rationals at x. R (R2) is recovered
from Excel's own GAMMALN(fl(1 + a0)) (probe aux column A) by inverting the final product(s): the doubles R with
fl(t1 R) == GL1 (and the R2 with fl(t1 fl(t2 R2)) == GL1). Candidates for g, each with every R in that set:
  raw    fl(h - LN(a0)), h = fl(a0 R) or fl(a0 fl(fl(a0 - 1) R2))  (kernel on a0 itself)
  gl     Excel's GAMMALN(a0) = fl(GL1 - LN(a0)) below 0.7
Scored as lgbest.py: a candidate is "in" when its g reaches the a0's ceiling (most cells exact, g within +-R ulps).
usage: rawt.py [R=4]"""
import json
import math
import sys
from collections import Counter, defaultdict

import pin
from lib import LN, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
RR = int(kw.get("R", "4"))
aux = json.loads((pin.lib.CASES / "t3570-r5-bpser-lg-aux.oracle.json").read_text())["sheets"]["Calc"]
meta = json.load(open(pin.HERE / "t3570-r5-bpser-lg.meta.json"))


def near(v, k=6):
    out = [v]
    for _ in range(k):
        out = [math.nextafter(out[0], -math.inf)] + out + [math.nextafter(out[-1], math.inf)]
    return out


def raw_hs(a0, gl1):
    x = 1.0 + a0
    t1 = sx("-", x, 1.0)
    if x <= 1.5:
        Rs = [r for r in near(gl1 / t1) if sx("*", t1, r) == gl1]
        return {sx("*", a0, r) for r in Rs}, len(Rs)
    t2 = sx("-", x, 2.0)
    R2s = [r for r in near(gl1 / (t1 * t2), 8) if sx("*", t1, sx("*", t2, r)) == gl1]
    am1 = sx("-", a0, 1.0)
    return {sx("*", a0, sx("*", am1, r)) for r in R2s}, len(R2s)


agg = defaultdict(Counter)
for (row, *_), p in zip(meta, pin.load()):
    a0 = p["a0"]
    if a0 >= 0.7:
        continue
    gl1 = float(aux[f"A{row}"]["v"])
    hs, nR = raw_hs(a0, gl1)
    graw = {sx("-", h, LN(a0)) for h in hs}
    g = p["gl"]
    for _ in range(RR):
        g = math.nextafter(g, -math.inf)
    score = {}
    for _ in range(2 * RR + 1):
        score[g] = sum(pin.predict(p, g, x) == e for x, e in p["cells"])
        g = math.nextafter(g, math.inf)
    top = max(score.values())
    best = {g for g, s in score.items() if s == top}
    g1 = sx("-", pin.K.gamln1(a0), LN(a0))
    key = (f"{a0:.2f}", "hold" if pin.hold(a0) else "tune")
    t = agg[key]
    t["n"] += 1
    t["R-found"] += nR > 0
    t["raw-ambig"] += len(graw) > 1
    t["gl"] += p["gl"] in best
    t["raw-all"] += bool(graw) and graw <= best
    t["raw-any"] += bool(graw & best)
    t["g1ln53"] += g1 in best
    t["raw!=gl"] += graw != {p["gl"]}
    t["raw!=gl&raw"] += graw != {p["gl"]} and bool(graw) and graw <= best
    t["raw!=gl&gl"] += graw != {p["gl"]} and p["gl"] in best
cols = ("n", "R-found", "raw-ambig", "gl", "raw-all", "raw-any", "g1ln53", "raw!=gl", "raw!=gl&raw", "raw!=gl&gl")
print("center split " + " ".join(f"{c:>11s}" for c in cols))
tot = defaultdict(Counter)
for key in sorted(agg):
    print(f"{key[0]:6s} {key[1]:5s} " + " ".join(f"{agg[key][c]:11d}" for c in cols))
    tot[key[1]].update(agg[key])
for sp in ("tune", "hold"):
    print(f"all    {sp:5s} " + " ".join(f"{tot[sp][c]:11d}" for c in cols))
