"""Kernel closed forms at a = 1 or b = 1 with brcomp-style logs, on every source that reaches them:
F (d1 = 2 or d2 = 2), T (df 2), BINOM (k = n-1 / k = 0), NEGBINOM (r = 1 / f = 0) and BETA.DIST.
Problem: I_x(a,b) with complement y (both doubles, formed by the caller).  Logs:
  ln(1-x) := log1p(-x) if x <= 0.375 else LN(y)        ln x := LN(x) if x <= 0.375 or y > 0.375 else log1p(-y)
  a == 1: P = EXP(b*ln(1-x)); lower = 0.5 - P + 0.5 (upper = P)
  b == 1: P = EXP(a*ln x);    lower = P (upper = 0.5 - P + 0.5)
log1p = correctly rounded (Excel's own log1p is 1 ulp off CR on some x in [0.15, 0.375], w1-density),
except log1p(-0.3) = -0.3566749439387324 read off BINOM (binom_log1p.py).
Variants: 'brcomp' (above), 'LNy' (always LN(y) / LN(x)), 'LN1mx' (LN(fl(1-x)))."""
import math, sys
from collections import defaultdict
import mpmath
sys.path.insert(0, '.')
import w5lib as L

L1P = {0.3: -0.3566749439387324}
def log1p_neg(t):
    if t in L1P: return L1P[t]
    return float(mpmath.log1p(-mpmath.mpf(t)))

def closed(x, y, a, b, var):
    if a == 1:
        if var == "brcomp": l = log1p_neg(x) if x <= 0.375 else L.LN(y)
        elif var == "LNy": l = L.LN(y)
        else: l = L.LN(1 - x)
        P = L.EXP(b * l); return (0.5 - P) + 0.5, P
    if b == 1:
        if var == "brcomp": l = L.LN(x) if (x <= 0.375 or y > 0.375) else log1p_neg(y)
        elif var == "LNy": l = L.LN(x)
        else: l = L.LN(x)
        P = L.EXP(a * l); return P, (0.5 - P) + 0.5
    return None

def items():
    """yield (source, x, y, a, b, side, excel) for closed-form shapes."""
    for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
        a_ = c["args"]
        if c["fn"] == "F.DIST" and a_[3] is not True: continue
        X, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
        if not X > 0 or (d1 != 2 and d2 != 2): continue
        p = d1 * X; w = 1 / (1 + d2 / p); v = 1 / (1 + p / d2)
        yield "F", c, w, v, d1 / 2, d2 / 2, "lower" if c["fn"] == "F.DIST" else "upper", c["excel"]
    for c in L.cells({"T.DIST.RT"}):
        t, n = c["args"][0], math.trunc(c["args"][1])
        if n != 2 or not t > 0: continue
        tt = t * t; s = n + tt; y = n / s; z = tt / s
        yield "T", c, y, z, 1.0, 0.5, "lower", 2 * c["excel"]
    for c in L.cells({"BINOM.DIST", "BINOMDIST"}):
        a_ = c["args"]
        if len(a_) < 4 or a_[3] is not True: continue
        k, n, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
        if not (0 < p < 1 and 0 <= k < n): continue
        if n - k == 1 or k == 0:
            yield "BINOM", c, 1 - p, p, float(n - k), float(k + 1), "lower", c["excel"]
    for c in L.cells({"NEGBINOM.DIST", "NEGBINOMDIST"}):
        a_ = c["args"]
        if len(a_) < 4 or a_[3] is not True: continue
        f, r, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
        if not (0 < p < 1 and f >= 0 and r >= 1): continue
        if r == 1 or f == 0:
            yield "NEGBINOM", c, p, 1 - p, float(r), float(f + 1), "lower", c["excel"]
    for c in L.cells({"BETA.DIST", "BETADIST"}):
        a_ = c["args"]
        if c["fn"] == "BETA.DIST":
            if len(a_) < 4 or a_[3] is not True or len(a_) > 4: continue
        elif len(a_) > 3: continue
        x, a, b = a_[0], a_[1], a_[2]
        if not (0 < x < 1) or (a != 1 and b != 1): continue
        yield "BETA.DIST", c, x, 1 - x, a, b, "lower", c["excel"]

if __name__ == "__main__":
    res = defaultdict(list)
    for src, c, x, y, a, b, side, ex in items():
        if a == 1 and b == 1: shape = "a=b=1"
        else: shape = "a=1" if a == 1 else "b=1"
        small = min(x, y) if False else x
        zone = "x<0.1" if x < 0.1 else ("x>0.5" if x > 0.5 else "0.1<=x<=0.5")
        for var in ("brcomp", "LNy", "LN1mx"):
            lo, up = closed(x, y, a, b, var)
            m = lo if side == "lower" else up
            res[(src, shape, zone, var)].append((m, ex))
    for k in sorted(res):
        print(k, L.fmt(L.score(res[k])))
