"""T-3570 r5-bpser-lg: does bpser keep GAMMALN's K3 kernel past x = 1.5? g = fl(K3(fl(1 + a0)) - LN(a0)) with K3 the
engine's port (gamma_kernels.rs K3_N/K3_D, FITTED, 0.322 exact on [0.7, 1.5]), every op stored.
Control: on a0 <= 0.5, where the worksheet uses K3, the share of a0 where this g equals Excel's GAMMALN(a0) measures
the port's fidelity at g's resolution. Test: on 0.5 < a0 < 0.7, k3ext against gl and g1ln53 in the a0's best g set.
usage: k3ext.py [R=4]"""
import math
import sys
from collections import Counter, defaultdict

import pin
from lib import LN, sx

N = [float.fromhex(h) for h in ("-0x1.59f97cb2592ffp-11", "-0x1.92587e5cc6b9cp-6", "-0x1.8558f6ae39a6bp-3",
     "-0x1.d3f137130667fp-2", "-0x1.4c00f86584772p-2", "0x1.59e7ca6931d79p-5", "0x1.6d0d491d3170ap-4",
     "0x1.3038d89824b68p-6", "0x1.c3aa7bf92db61p-11", "0x1.337aa15bf7e5bp-19")]
D = [float.fromhex(h) for h in ("0x1.0000000000000p-13", "0x1.f168c3e77cf05p-8", "0x1.72eb273a4ee6dp-4",
     "0x1.6b0b4a0843b68p-2", "0x1.18e8390e537a6p-1", "0x1.6e6b1d23cc97ep-2", "0x1.8a4a651d2c4f4p-4",
     "0x1.35398ed10925ep-7", "0x1.ebfe98d77f93fp-13", "-0x1.460d42bdb1be4p-31")]


def horner(cs, x):
    s = cs[-1]
    for c in reversed(cs[:-1]):
        s = sx("+", sx("*", s, x), c)
    return s


def k3(x):
    return sx("*", x - 1.0, sx("/", horner(N, x), horner(D, x)))


kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "4"))
agg = defaultdict(Counter)
for p in pin.load():
    a0 = p["a0"]
    if a0 >= 0.7:
        continue
    gk = sx("-", k3(1.0 + a0), LN(a0))
    if 1.0 + a0 <= 1.5:
        t = agg[("control a0<=0.5", f"{a0:.2f}")]
        t["n"] += 1
        t["k3==gl"] += gk == p["gl"]
        continue
    g = p["gl"]
    for _ in range(R):
        g = math.nextafter(g, -math.inf)
    score = {}
    for _ in range(2 * R + 1):
        score[g] = sum(pin.predict(p, g, x) == e for x, e in p["cells"])
        g = math.nextafter(g, math.inf)
    top = max(score.values())
    best = {g for g, s in score.items() if s == top}
    g1 = sx("-", pin.K.gamln1(a0), LN(a0))
    t = agg[("test 0.5<a0<0.7", f"{a0:.2f}")]
    t["n"] += 1
    for nm, v in (("gl", p["gl"]), ("g1ln53", g1), ("k3ext", gk)):
        t[nm] += v in best
    t["k3!=gl"] += gk != p["gl"]
    t["k3!=gl&k3"] += gk != p["gl"] and gk in best
    t["k3!=gl&gl"] += gk != p["gl"] and p["gl"] in best
for k in sorted(agg):
    print(k[0], k[1], dict(agg[k]))
