"""T.DIST.RT at df 2 through the kernel closed forms: which y = f(t) (and complement) does T pass?
Model: L = I_y(1, 1/2)/2.  Oriented: y <= 0.5 -> a0 = 1, x0 = y: w1 = EXP(0.5*LN(fl(1-y))) (computed upper), I = 1 - w1;
y > 0.5 -> swapped a0 = 1/2, b0 = 1, x0 = c (complement): w = EXP(0.5*LN(c)) = upper of the original, I = 1 - w."""
import math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD
res = defaultdict(list)
for c in L.cells({"T.DIST.RT"}):
    t, n = c["args"][0], float(math.trunc(c["args"][1]))
    if n != 2 or not t > 0: continue
    tt = t * t
    forms = {
        "n/(n+tt) | tt/(n+tt)": (n / (n + tt), tt / (n + tt)),
        "n/(n+tt) | 1-y": (n / (n + tt), 1 - n / (n + tt)),
        "1/(1+tt/n) | 1-y": (1 / (1 + tt / n), 1 - 1 / (1 + tt / n)),
        "1/(1+tt/n) | 1/(1+n/tt)": (1 / (1 + tt / n), 1 / (1 + n / tt)),
        "ext n/(n+tt) | 1-y": (float(LD(n) / (LD(n) + LD(t) * LD(t))), 1 - float(LD(n) / (LD(n) + LD(t) * LD(t)))),
    }
    for k, (y, cc) in forms.items():
        if y < 0.1: continue
        if y <= 0.5:
            P = L.EXP(0.5 * L.LN(1 - y)); I = 1 - P; zone = "y<=0.5"
        else:
            P = L.EXP(0.5 * L.LN(cc)); I = 1 - P; zone = "y>0.5"
        res[(zone, k)].append((I / 2, c["excel"]))
for k in sorted(res): print(k, L.fmt(L.score(res[k])))
