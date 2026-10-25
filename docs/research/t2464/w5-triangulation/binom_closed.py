"""BINOM.DIST / NEGBINOM.DIST at the incomplete-beta closed-form points.
BINOM cdf: P(X<=k) = I_{1-p}(n-k, k+1).  k=0: q^n (b=1).  k=n-1: 1-p^n (a=1).  k=n: 1.
pmf k=0: q^n; pmf k=n: p^n.
NEGBINOM (f failures, r successes, p): cdf P(F<=f) = I_p(r, f+1).  f=0: p^r (b=1).  r=1: 1-q^(f+1) (a=1).
pmf f=0: p^r."""
import math, sys
from collections import defaultdict
from fractions import Fraction
sys.path.insert(0, '.')
import w5lib as L

def frac_pow(base, e):
    return float(Fraction(base) ** int(e))

res = defaultdict(list)
for c in L.cells({"BINOM.DIST", "BINOMDIST", "NEGBINOM.DIST", "NEGBINOMDIST"}):
    a = c["args"]
    ex = c["excel"]
    if c["fn"].startswith("BINOM"):
        if len(a) < 4: continue
        k, n, p, cum = math.trunc(a[0]), math.trunc(a[1]), a[2], a[3]
        cum = bool(cum)
        if not (0 <= k <= n and 0 < p < 1): continue
        q = 1 - p
        cands = {}
        if k == 0:
            e = n * L.LN(q)
            cands["EXP(n*LN(1-p))"] = L.EXP(e)
            cands["exact q^n (q=fl(1-p))"] = frac_pow(q, n)
            cands["exact (1-p)^n, p decimal"] = float((1 - Fraction(repr(p))) ** n)
            cands["pow q^n libm"] = q ** n
            res[(c["fn"], "cdf" if cum else "pmf", "k=0")].append((cands, ex))
        if k == n and not cum:
            cands = {}
            cands["EXP(n*LN(p))"] = L.EXP(n * L.LN(p))
            cands["exact p^n"] = frac_pow(p, n)
            cands["pow"] = p ** n
            res[(c["fn"], "pmf", "k=n")].append((cands, ex))
        if k == n - 1 and cum and n >= 1:
            cands = {}
            P = L.EXP(n * L.LN(p))
            cands["1-EXP(n*LN(p))"] = 1 - P
            cands["0.5-P+0.5"] = (0.5 - P) + 0.5
            cands["exact 1-p^n"] = float(1 - Fraction(p) ** n)
            res[(c["fn"], "cdf", "k=n-1")].append((cands, ex))
    else:
        if len(a) < 3: continue
        f, r, p = math.trunc(a[0]), math.trunc(a[1]), a[2]
        cum = bool(a[3]) if len(a) > 3 else False
        if not (f >= 0 and r >= 1 and 0 < p < 1): continue
        cands = {}
        if f == 0:
            cands["EXP(r*LN(p))"] = L.EXP(r * L.LN(p))
            cands["exact p^r"] = frac_pow(p, r)
            cands["pow"] = p ** r
            res[(c["fn"], "cdf" if cum else "pmf", "f=0")].append((cands, ex))
        if r == 1 and cum:
            cands = {}
            q = 1 - p
            P = L.EXP((f + 1) * L.LN(q))
            cands["1-EXP((f+1)*LN(1-p))"] = 1 - P
            cands["exact 1-q^(f+1)"] = float(1 - Fraction(q) ** (f + 1))
            res[(c["fn"], "cdf", "r=1")].append((cands, ex))
for key in sorted(res):
    rows = res[key]
    print("==", key, len(rows))
    for name in rows[0][0]:
        print(f"   {name:28s} {L.fmt(L.score([(r[0][name], r[1]) for r in rows]))}")
