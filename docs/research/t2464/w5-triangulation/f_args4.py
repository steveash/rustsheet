"""Robust loss per argument form, per side (small argument = w if w <= 0.5 else v); exact tail at the form's small
argument vs Excel's directly computed tail.  Loss = sum min(|ulp err|, 50); also counts within 1 ulp."""
import math, sys
from collections import defaultdict
import numpy as np
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD
from dispatch import bratio_branch

def forms(x, d1, d2):
    p = d1 * x; s = p + d2; r = d2 / d1; u = p / d2; q = d2 / p
    pe = LD(d1) * LD(x); se = pe + LD(d2)
    return {
        "p/s | d2/s": (p / s, d2 / s),
        "ext": (float(pe / se), float(LD(d2) / se)),
        "x/(x+r) | r/(x+r)": (x / (x + r), r / (x + r)),
        "1/(1+q) | 1/(1+u)": (1 / (1 + q), 1 / (1 + u)),
        "u/(1+u) | q/(1+q)": (u / (1 + u), q / (1 + q)),
        "u/(1+u) | 1/(1+u)": (u / (1 + u), 1 / (1 + u)),
        "1/(1+q) | q/(1+q)": (1 / (1 + q), q / (1 + q)),
        "x*d1/(x*d1+d2)": ((x * d1) / (x * d1 + d2), d2 / (x * d1 + d2)),
        "x/(x+d2/d1) | 1-": (x / (x + d2 / d1), 1 / (1 + x * (d1 / d2))),
    }
side_rows = defaultdict(list)
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    x, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not (x > 0 and d1 >= 1 and d2 >= 1) or d1 > 1e10 or d2 > 1e10: continue
    ex = c["excel"]
    if ex in (0.0, 1.0): continue
    a, b = d1 / 2, d2 / 2
    F = forms(x, d1, d2)
    w0, v0 = F["p/s | d2/s"]
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    br, ind, comp = bratio_branch(a, b, w0, v0)
    if tail != comp: continue
    side = "w" if w0 <= 0.5 else "v"
    errs = {}
    for k, (w, v) in F.items():
        if side == "w":
            t = L.ibeta(w, a, b); val = t if tail == "lower" else 1 - t
        else:
            t = L.ibeta(v, b, a); val = t if tail == "upper" else 1 - t
        errs[k] = abs(float(val) - ex) / math.ulp(ex)
    if max(errs.values()) - min(errs.values()) < 3: continue
    side_rows[side].append(errs)
L.save_cache()
for side, rows in side_rows.items():
    print(f"== small argument {side}: {len(rows)} discriminating cells")
    for k in rows[0]:
        e = np.array([r[k] for r in rows])
        print(f"  {k:22s} loss {np.minimum(e, 50).sum():8.0f}  <=1 {(e<=1).sum():4d}  <=4 {(e<=4).sum():4d}")
