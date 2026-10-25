"""Which beta argument does F use?  For every F.DIST(TRUE)/F.DIST.RT/FDIST cell, compare Excel with the exact
tail at several candidate arguments.  The tail is evaluated as a function of the one argument the candidate names:
  lower tails: I_w(a,b) at w candidates; upper tails: I_v(b,a) at v candidates (v = complement argument).
Reports, per region, the count of cells within 1 / 4 ulp of each candidate and the median |ulp|."""
import math, sys
from collections import defaultdict
import numpy as np
import mpmath
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD
from dispatch import bratio_branch

def wcands(x, d1, d2):
    p = d1 * x; s = p + d2
    w = p / s; v = d2 / s
    pe = LD(d1) * LD(x); se = pe + LD(d2)
    we, ve = pe / se, LD(d2) / se
    return {
        "w=p/s": w, "w=1-v": 1 - v, "w=ext": float(we), "w=x/(x+d2/d1)": x / (x + d2 / d1),
        "w=1/(1+d2/p)": 1 / (1 + d2 / p),
    }, {
        "v=d2/s": v, "v=1-w": 1 - w, "v=ext": float(ve), "v=(d2/d1)/(x+d2/d1)": (d2 / d1) / (x + d2 / d1),
        "v=1/(1+p/d2)": 1 / (1 + p / d2),
    }

stats = defaultdict(lambda: defaultdict(list))
n = 0
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    x, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not (x > 0 and d1 >= 1 and d2 >= 1) or d1 > 1e10 or d2 > 1e10: continue
    if c["excel"] in (0.0, 1.0): continue
    a, b = d1 / 2, d2 / 2
    W, V = wcands(x, d1, d2)
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    br, ind, comp = bratio_branch(a, b, W["w=p/s"], V["v=d2/s"])
    if tail != comp:
        continue  # only directly computed tails
    region = ("w<=0.5" if W["w=p/s"] <= 0.5 else "w>0.5") + "," + tail
    ex = c["excel"]
    n += 1
    for name, t in list(W.items()) + list(V.items()):
        if not (0 < t < 1): continue
        if tail == "lower":
            val = L.ibeta(t, a, b) if name.startswith("w") else 1 - L.ibeta(t, b, a)
        else:
            val = 1 - L.ibeta(t, a, b) if name.startswith("w") else L.ibeta(t, b, a)
        u = abs(float(val) - ex) / math.ulp(ex)
        stats[region][name].append(u)
L.save_cache()
print("cells (directly computed tails)", n)
for region in sorted(stats):
    print("==", region)
    for name, us in sorted(stats[region].items(), key=lambda kv: np.median(kv[1])):
        us = np.array(us)
        print(f"  {name:22s} n={len(us):4d} <=1:{(us<=1).sum():4d} <=4:{(us<=4).sum():4d} median {np.median(us):8.2f} p90 {np.percentile(us,90):10.1f}")
