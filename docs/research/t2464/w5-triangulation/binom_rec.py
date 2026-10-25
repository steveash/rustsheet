"""BINOM.DIST cdf as a running sum of recursively generated pmf terms starting from t0 = EXP(n*l(p)), l = ln(1-p)
(log1p for p <= 0.375, LN(1-p) above; l(0.3) pinned to Excel's value from binom_log1p.py), and pmf(k) compared too."""
import math, sys
from collections import defaultdict, Counter
import mpmath
import numpy as np
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD

LOG1P = {0.3: -0.3566749439387324}
def lq(p):
    if p in LOG1P: return LOG1P[p]
    if p <= 0.375: return float(mpmath.log1p(-mpmath.mpf(p)))
    return L.LN(1 - p)

pmf, cdf = {}, {}
for c in L.cells({"BINOM.DIST"}):
    a = c["args"]
    if len(a) < 4 or not isinstance(a[3], bool): continue
    k, n, p, cum = math.trunc(a[0]), math.trunc(a[1]), a[2], a[3]
    if 0 <= k <= n and 0 < p < 1:
        (cdf if cum else pmf)[(n, p, k)] = c["excel"]
groups = defaultdict(dict)
for (n, p, k), v in cdf.items(): groups[(n, p)][k] = v

def terms(n, p, form):
    q = 1 - p
    t = L.EXP(n * lq(p))
    out = [t]
    r = p / q
    for k in range(n):
        if form == "R1": t = t * (n - k) / (k + 1) * r
        elif form == "R2": t = t * (((n - k) * p) / ((k + 1) * q))
        elif form == "R3": t = t * ((n - k) / (k + 1)) * r
        elif form == "R4": t = t * (n - k) * p / ((k + 1) * q)
        elif form == "R5": t = t * r * (n - k) / (k + 1)
        out.append(t)
    return out

for form in ("R1", "R2", "R3", "R4", "R5"):
    sc = Counter(); scp = Counter()
    for (n, p), d in groups.items():
        if n > 200: continue
        T = terms(n, p, form)
        s = 0.0
        for k in range(n + 1):
            s = s + T[k]
            band = "p<=.5" if p <= 0.5 else "p>.5"
            if k in d:
                sc[(band, s == d[k])] += 1
            if (n, p, k) in pmf:
                scp[(band, T[k] == pmf[(n, p, k)])] += 1
    print(form, "cdf", dict(sorted(sc.items())), " pmf", dict(sorted(scp.items())))
