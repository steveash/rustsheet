"""F's argument rounding, decided on the kernel closed forms (oriented problem: x0 <= 0.5):
  b0 == 1: EXP(a0*LN(x0));  a0 == 1 and x0 >= 0.1: (0.5 - EXP(b0*LN(fl(1-x0)))) + 0.5.
x0 is w (w <= 0.5) or v (w > 0.5); every form of w and v is scored on the cells whose oriented shapes hit a closed form."""
import math, sys
from collections import defaultdict
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD

def forms(x, d1, d2):
    p = d1 * x; s = p + d2; r = d2 / d1; u = p / d2; q = d2 / p
    pe = LD(d1) * LD(x); se = pe + LD(d2)
    return {
        "p/s | d2/s": (p / s, d2 / s),
        "ext": (float(pe / se), float(LD(d2) / se)),
        "x/(x+r) | r/(x+r)": (x / (x + r), r / (x + r)),
        "1/(1+q) | 1/(1+u)": (1 / (1 + q), 1 / (1 + u)),
        "u/(1+u) | q/(1+q)": (u / (1 + u), q / (1 + q)),
        "1-v | 1-w": (1 - d2 / s, 1 - p / s),
        "1/(1+r/x) | 1/(1+x/r)": (1 / (1 + r / x), 1 / (1 + x / r)),
    }

def oriented_closed(a, b, x, y):
    """lower tail of the original problem, or None if no closed form"""
    swap = x > 0.5
    a0, b0, x0 = (b, a, y) if swap else (a, b, x)
    if b0 == 1:
        w = L.EXP(a0 * L.LN(x0))
    elif a0 == 1 and x0 >= 0.1:
        w = (0.5 - L.EXP(b0 * L.LN(1 - x0))) + 0.5
    else:
        return None
    return ((0.5 - w) + 0.5) if swap else w, w, swap

res = defaultdict(list)
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a_ = c["args"]
    if c["fn"] == "F.DIST" and a_[3] is not True: continue
    X, d1, d2 = a_[0], float(math.trunc(a_[1])), float(math.trunc(a_[2]))
    if not X > 0 or (d1 != 2 and d2 != 2): continue
    a, b = d1 / 2, d2 / 2
    tail = "lower" if c["fn"] == "F.DIST" else "upper"
    F = forms(X, d1, d2)
    ref = oriented_closed(a, b, *F["1/(1+q) | 1/(1+u)"])
    if ref is None: continue
    for k, (w, v) in F.items():
        if not (0 < w < 1 and 0 < v < 1): continue
        o = oriented_closed(a, b, w, v)
        if o is None: continue
        low, w0, swap = o
        computed_lower = not swap
        val = low if tail == "lower" else (w0 if swap else (0.5 - w0) + 0.5)
        grp = ("b0=1" if (b if not swap else a) == 1 else "a0=1") + (",x0=v" if swap else ",x0=w") + "," + tail
        res[(grp, k)].append((val, c["excel"]))
for key in sorted(res): print(key, L.fmt(L.score(res[key])))
