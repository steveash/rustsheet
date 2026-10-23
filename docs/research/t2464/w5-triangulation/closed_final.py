"""Final score of the kernel closed-form rule on every source (oriented problem, x0 <= 0.5 after bratio's x > 0.5 swap):
  b0 == 1               : w  = EXP(a0*LN(x0))                 (computed: oriented lower), w1 = (0.5 - w) + 0.5
  a0 == 1 and x0 >= 0.1 : w1 = EXP(b0*LN(fl(1 - x0)))         (computed: oriented upper), w  = (0.5 - w1) + 0.5
  (a0 == 1 and x0 < 0.1: not a closed form; Excel is near the exact value there)
Arguments: F x = 1/(1+d2/(d1 X)), y = 1/(1+d1 X/d2); T(df 2) x = fl(2/fl(2+t*t)), y = fl(1-x); BINOM x = p, y = fl(1-p),
I_p(k+1, n-k) upper (k = 0 excluded: BINOM's own pmf, binom_log1p.py); NEGBINOM x = p, y = fl(1-p); BETA.DIST y = fl(1-x).
Reports all / tune half / hold-out half (md5 of case+cell)."""
import math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L


import os
CUT = os.environ.get("CUT", "y0<=0.9")


def a1_cut(x0, y0):
    if CUT == "x0>=0.1":
        return x0 >= 0.1
    if CUT == "x0>=1-0.9":
        return x0 >= 0.09999999999999998
    return y0 <= 0.9


def closed_value(a, b, x, y, side):
    swap = x > 0.5
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 == 1:
        w = L.EXP(a0 * L.LN(x0)); w1 = (0.5 - w) + 0.5
    elif a0 == 1 and a1_cut(x0, y0):
        w1 = L.EXP(b0 * L.LN(1 - x0)); w = (0.5 - w1) + 0.5
    else:
        return None
    lower, upper = (w1, w) if swap else (w, w1)
    return lower if side == "lower" else upper


def items():
    for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
        a_ = c["args"]
        if c["fn"] == "F.DIST" and a_[3] is not True: continue
        X, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
        if not X > 0 or (d1 != 2 and d2 != 2): continue
        p = d1 * X
        yield "F", c, 1 / (1 + d2 / p), 1 / (1 + p / d2), d1 / 2, d2 / 2, "lower" if c["fn"] == "F.DIST" else "upper", c["excel"]
    for c in L.cells({"T.DIST.RT"}):
        t, n = c["args"][0], math.trunc(c["args"][1])
        if n != 2 or not t > 0: continue
        y = 2.0 / (2.0 + t * t)
        yield "T.DIST.RT(df2)", c, y, 1 - y, 1.0, 0.5, "lower", 2 * c["excel"]
    for c in L.cells({"BINOM.DIST", "BINOMDIST"}):
        a_ = c["args"]
        if len(a_) < 4 or a_[3] is not True: continue
        k, n, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
        if not (0 < p < 1 and 1 <= k < n): continue
        yield "BINOM", c, p, 1 - p, float(k + 1), float(n - k), "upper", c["excel"]
    for c in L.cells({"NEGBINOM.DIST"}):
        a_ = c["args"]
        if len(a_) < 4 or a_[3] is not True: continue
        f, r, p = math.trunc(a_[0]), math.trunc(a_[1]), a_[2]
        if not (0 < p < 1 and f >= 0 and r >= 1): continue
        yield "NEGBINOM", c, p, 1 - p, float(r), float(f + 1), "lower", c["excel"]
    for c in L.cells({"BETA.DIST", "BETADIST"}):
        a_ = c["args"]
        if c["fn"] == "BETA.DIST" and (len(a_) != 4 or a_[3] is not True): continue
        if c["fn"] == "BETADIST" and len(a_) != 3: continue
        x, a, b = a_[0], a_[1], a_[2]
        if not (0 < x < 1): continue
        yield "BETA.DIST", c, x, 1 - x, a, b, "lower", c["excel"]


if __name__ == "__main__":
    res = defaultdict(lambda: defaultdict(list))
    for src, c, x, y, a, b, side, ex in items():
        m = closed_value(a, b, x, y, side)
        if m is None: continue
        swap = x > 0.5
        a0, b0 = (b, a) if swap else (a, b)
        kind = "b0=1" if b0 == 1 else "a0=1"
        h = L.holdout(c["case"], c["cell"])
        for key in ((src, kind), ("ALL", kind), ("ALL", "all")):
            res[key]["all"].append((m, ex)); res[key]["hold" if h else "tune"].append((m, ex))
    for key in sorted(res):
        d = res[key]
        print(f"{str(key):28s} all {L.fmt(L.score(d['all'])):22s} tune {L.fmt(L.score(d['tune'])):22s} hold-out {L.fmt(L.score(d['hold']))}")
    # the a0 = 1, x0 < 0.1 cells: closed form vs exact
