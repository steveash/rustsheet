"""T-3570 r5-bpser-lg: per a0, cells exact with each lnGamma(a0) candidate (pin.cands: gl, g1ln53, acc) in bpser's L,
and the per-a0 winner on the a0 where the candidates' L differ. Tune and hold-out halves (md5 of a0) separately.
usage: lgwin.py"""
from collections import Counter, defaultdict

import pin
from lib import sx

NAMES = ("gl", "g1ln53", "acc")
agg = defaultdict(Counter)
for p in pin.load():
    cs = pin.cands(p)
    Ls = {nm: sx("+", g, sx("-", p["glb"], p["glab"])) for nm, g in cs.items()}
    hits = {nm: sum(pin.predict(p, cs[nm], x) == e for x, e in p["cells"]) for nm in NAMES}
    key = (f"{p['a0']:.2f}", "hold" if pin.hold(p["a0"]) else "tune")
    t = agg[key]
    t["n"] += 1
    for nm in NAMES:
        t[nm] += hits[nm]
    if Ls["gl"] != Ls["g1ln53"]:
        t["Ldiff"] += 1
        if hits["gl"] > hits["g1ln53"]:
            t["gl>"] += 1
        elif hits["gl"] < hits["g1ln53"]:
            t["g1ln>"] += 1
        else:
            t["tie"] += 1
cols = ("n", "gl", "g1ln53", "acc", "Ldiff", "gl>", "g1ln>", "tie")
print("center split " + " ".join(f"{c:>7s}" for c in cols) + "   (gl/g1ln53/acc: exact cells of 8n)")
tot = defaultdict(Counter)
for key in sorted(agg):
    print(f"{key[0]:6s} {key[1]:5s} " + " ".join(f"{agg[key][c]:7d}" for c in cols))
    tot[key[1]].update(agg[key])
for sp in ("tune", "hold"):
    print(f"all    {sp:5s} " + " ".join(f"{tot[sp][c]:7d}" for c in cols))
