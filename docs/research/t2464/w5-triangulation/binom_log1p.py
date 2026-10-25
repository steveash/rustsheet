"""Read Excel's ln(1-p) from BINOM.DIST(0,n,p) = EXP(n * l) over all n: scan doubles l near log1p(-p)."""
import math, sys
from collections import defaultdict
import mpmath
sys.path.insert(0, '.')
import w5lib as L
rows = defaultdict(dict)
for c in L.cells({"BINOM.DIST", "BINOMDIST"}):
    a = c["args"]
    if len(a) < 4: continue
    k, n, p = math.trunc(a[0]), math.trunc(a[1]), a[2]
    if k == 0 and 0 < p < 1 and n >= 1:
        rows[p][n] = c["excel"]
for v in [x for x in rows]:
    pass
for p in sorted(rows):
    l0 = float(mpmath.log1p(-mpmath.mpf(p)))
    cands = [l0]
    for i in range(30):
        cands.append(math.nextafter(cands[-1], math.inf))
    lo = [l0]
    for i in range(30):
        lo.append(math.nextafter(lo[-1], -math.inf))
    cands = sorted(set(cands + lo))
    ns = sorted(rows[p])
    allexp = [fl * n for fl in cands for n in ns]
    L.xl("EXP", allexp)
    best = []
    for l in cands:
        ok = sum(L.EXP(l * n) == rows[p][n] for n in ns)
        best.append((ok, l))
    best.sort(reverse=True)
    lnq = L.LN(1 - p)
    top = best[0]
    print(f"p={p}: {len(ns)} n values; best l={top[1]!r} ({(top[1]-l0)/math.ulp(l0):+.0f} ulp from CR log1p) fits {top[0]}; "
          f"LN(1-p) is {(lnq-l0)/math.ulp(l0):+.0f} ulp (fits {sum(L.EXP(lnq*n)==rows[p][n] for n in ns)}); next best {best[1][0]} at {(best[1][1]-l0)/math.ulp(l0):+.0f}")
