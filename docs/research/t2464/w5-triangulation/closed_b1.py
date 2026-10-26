"""b = 1, x > 0.5 and a = 1 split finer by the complement y; ln x as LN(x), log1p(-y) (CR), LN(fl(1-y))."""
import math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L
from closed_kernel import items, log1p_neg
res = defaultdict(list)
for src, c, x, y, a, b, side, ex in items():
    if b == 1 and a != 1 and x > 0.5:
        band = "y<0.1" if y < 0.1 else ("y<=0.375" if y <= 0.375 else "y>0.375")
        for name, l in (("LN(x)", L.LN(x)), ("log1p(-y)", log1p_neg(y)), ("LN(1-y)", L.LN(1 - y))):
            P = L.EXP(a * l)
            m = P if side == "lower" else (0.5 - P) + 0.5
            res[(src, "b=1,x>0.5", band, side, name)].append((m, ex))
    if a == 1 and b != 1 and x < 0.1:
        for name, l in (("log1p(-x)", log1p_neg(x)), ("LN(y)", L.LN(y)), ("LN(1-x)", L.LN(1 - x))):
            P = L.EXP(b * l)
            m = (0.5 - P) + 0.5 if side == "lower" else P
            res[(src, "a=1,x<0.1", "", side, name)].append((m, ex))
        res[(src, "a=1,x<0.1", "", side, "exact")].append((float(L.ibeta(x, a, b)) if side == "lower" else float(1 - L.ibeta(x, a, b)), ex))
for k in sorted(res): print(k, L.fmt(L.score(res[k])))
L.save_cache()
