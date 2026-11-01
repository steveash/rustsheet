"""T-3570 r5-bpser-lg: per a0, the set of g doubles (within +-R ulps of Excel's GAMMALN(a0)) whose L reaches the
a0's ceiling (most of its 8 cells exact), and whether each candidate lnGamma(a0) form is in that set. Candidates use
Excel's own values from probe t3570-r5-bpser-lg-aux where they need GAMMA or GAMMALN off a0:
  gl       GAMMALN(a0)                          g1ln53  fl(gamln1(a0) - LN(a0))   (TOMS, double code)
  acc      lnGamma(a0) correctly rounded        glp1    fl(GAMMALN(fl(1 + a0)) - LN(a0))
  lng      LN(GAMMA(a0)) (Excel's, column C)    lng1    fl(LN(GAMMA(fl(1 + a0))) - LN(a0))
  lngdiv   LN(fl(GAMMA(fl(1 + a0)) / a0))
  hacc     fl(lnGamma(1 + a0) correctly rounded - LN(a0))   g1ln64  fl53(fl64(gamln1(a0) - LN(a0)))
"best set" is the g set at the ceiling; a candidate is "in" when its g is in it; "sole" counts a0 where the
candidate is in and gl is out or vice versa (only those separate it from gl).
usage: lgbest.py [R=4]"""
import json
import math
import sys
from collections import Counter, defaultdict

import pin
from lib import LN, sx

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
R = int(kw.get("R", "4"))
aux = json.loads((pin.lib.CASES / "t3570-r5-bpser-lg-aux.oracle.json").read_text())["sheets"]["Calc"]
meta = json.load(open(pin.HERE / "t3570-r5-bpser-lg.meta.json"))
NAMES = ("gl", "g1ln53", "acc", "glp1", "hacc", "g1ln64", "lng", "lng1", "lngdiv")
agg = defaultdict(Counter)
for (row, *_), p in zip(meta, pin.load()):
    a0 = p["a0"]

    def v(col):
        return float(aux[f"{col}{row}"]["v"])

    cs = pin.cands(p)
    cs["glp1"] = sx("-", v("A"), LN(a0))
    with pin.mpmath.workdps(40):
        h = pin.mpmath.loggamma(pin.mpf(a0) + 1)
    cs["hacc"] = sx("-", float(h), LN(a0))
    cs["g1ln64"] = float(pin.lib.fl53(pin.lib.fl64(pin.mpf(pin.K.gamln1(a0)) - pin.mpf(LN(a0)))))
    t0 = agg[(f"{a0:.2f}", "ident")]
    t0["n"] += 1
    t0["gl==glp1"] += cs["glp1"] == p["gl"]
    cs["lng"] = v("C")
    cs["lng1"] = sx("-", v("E"), LN(a0))
    cs["lngdiv"] = LN(sx("/", v("D"), a0))
    g = p["gl"]
    for _ in range(R):
        g = math.nextafter(g, -math.inf)
    score = {}
    for _ in range(2 * R + 1):
        score[g] = sum(pin.predict(p, g, x) == e for x, e in p["cells"])
        g = math.nextafter(g, math.inf)
    top = max(score.values())
    best = {g for g, s in score.items() if s == top}
    key = (f"{a0:.2f}", "hold" if pin.hold(a0) else "tune")
    t = agg[key]
    t["n"] += 1
    t["top"] += top
    for nm in NAMES:
        inn = cs[nm] in best
        t[nm] += inn
        if nm != "gl" and inn != (cs["gl"] in best):
            t[f"{nm}+" if inn else f"{nm}-"] += 1
cols = ["n", "top"] + list(NAMES)
print("center split " + " ".join(f"{c:>6s}" for c in cols) + "  | vs gl: +in&gl-out / -out&gl-in")
tot = defaultdict(Counter)
for key in sorted(k for k in agg if k[1] != "ident"):
    t = agg[key]
    vs = " ".join(f"{nm}:{t[nm + '+']}/{t[nm + '-']}" for nm in NAMES[1:])
    print(f"{key[0]:6s} {key[1]:5s} " + " ".join(f"{t[c]:6d}" for c in cols) + "  | " + vs)
    tot[key[1]].update(t)
for sp in ("tune", "hold"):
    t = tot[sp]
    vs = " ".join(f"{nm}:{t[nm + '+']}/{t[nm + '-']}" for nm in NAMES[1:])
    print(f"all    {sp:5s} " + " ".join(f"{t[c]:6d}" for c in cols) + "  | " + vs)
for key in sorted(k for k in agg if k[1] == "ident"):
    print(f"identity GAMMALN(a0) == fl(GAMMALN(fl(1 + a0)) - LN(a0)) at a0~{key[0]}: {agg[key]['gl==glp1']}/{agg[key]['n']}")
