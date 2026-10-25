"""F.DIST / F.DIST.RT / FDIST at d1 = 2 (a = 1) or d2 = 2 (b = 1): which argument rounding and which closed form."""
import itertools, math, sys
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD

rows = []
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a = c["args"]
    x, d1, d2 = a[0], math.trunc(a[1]), math.trunc(a[2])
    if c["fn"] == "F.DIST":
        if a[3] is not True:
            continue
        kind = "cdf"
    else:
        kind = "rt"
    if not (x > 0 and d1 >= 1 and d2 >= 1) or (d1 != 2 and d2 != 2):
        continue
    rows.append((c, kind, x, float(d1), float(d2)))
print(len(rows), "cells")

def args(x, d1, d2, form):
    if form == "dbl":
        p = d1 * x; s = p + d2; return p / s, d2 / s
    if form == "x87":
        p = L.x87('*', d1, x); s = L.x87('+', p, d2); return L.x87('/', p, s), L.x87('/', d2, s)
    if form == "ext":
        p = LD(d1) * LD(x); s = p + LD(d2); return float(p / s), float(LD(d2) / s)
    if form == "ratio":
        r = d2 / d1; s = x + r; return x / s, r / s
    if form == "inv":   # w = 1/(1 + d2/(d1 x))
        q = d2 / (d1 * x); return 1 / (1 + q), q / (1 + q)
    raise ValueError

FORMS = ["dbl", "x87", "ext", "ratio", "inv"]
# collect LN/EXP inputs
need_ln = set()
for c, kind, x, d1, d2 in rows:
    for f in FORMS:
        w, v = args(x, d1, d2, f)
        need_ln |= {w, v, 1 - w, 1 - v}
need_ln = [t for t in need_ln if t > 0]
L.xl("LN", need_ln)

def lnx(t):
    return L.xl("LN", [t])[0]

def models(x, d1, d2, kind, f):
    w, v = args(x, d1, d2, f)
    a, b = d1 / 2, d2 / 2
    out = {}
    # candidate exponent args, then EXP batch done lazily
    if d1 == 2:  # a = 1: I_w(1,b) = 1 - (1-w)^b ; upper = v^b
        for yname, y in (("v", v), ("1-w", 1 - w)):
            e = b * lnx(y) if y > 0 else -math.inf
            out[f"P=EXP(b*LN({yname}))"] = e
    if d2 == 2:  # b = 1: I_w(a,1) = w^a ; upper = 1 - w^a
        for yname, y in (("w", w), ("1-v", 1 - v)):
            e = a * lnx(y) if y > 0 else -math.inf
            out[f"Q=EXP(a*LN({yname}))"] = e
    return out

exps = set()
tab = []
for c, kind, x, d1, d2 in rows:
    for f in FORMS:
        m = models(x, d1, d2, kind, f)
        tab.append((c, kind, x, d1, d2, f, m))
        exps |= {e for e in m.values() if math.isfinite(e)}
L.xl("EXP", list(exps))
def E(e):
    return 0.0 if e == -math.inf else L.xl("EXP", [e])[0]

from collections import defaultdict
res = defaultdict(list)
for c, kind, x, d1, d2, f, m in tab:
    for k, e in m.items():
        P = E(e)
        if k.startswith("P"):   # a = 1
            cdf, rt = 1 - P, P
        else:
            cdf, rt = P, 1 - P
        val = cdf if kind == "cdf" else rt
        grp = ("a=1" if d1 == 2 else "b=1") + ("" if not (d1 == 2 and d2 == 2) else "(both)")
        res[(kind, grp, f, k)].append((val, c["excel"]))
for key in sorted(res):
    print(key, L.fmt(L.score(res[key])))
L.save_cache()
