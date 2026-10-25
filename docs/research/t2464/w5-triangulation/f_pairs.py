"""F.DIST(x,d1,d2,TRUE) vs F.DIST.RT(x,d1,d2) at the same point: which is the complement of which."""
import math, sys
from collections import Counter, defaultdict
sys.path.insert(0, '.')
import w5lib as L
cdf, rt = {}, {}
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a = c["args"]
    if c["fn"] == "F.DIST" and a[3] is not True: continue
    key = (a[0], math.trunc(a[1]), math.trunc(a[2]))
    (cdf if c["fn"] == "F.DIST" else rt)[key] = c["excel"]
both = sorted(set(cdf) & set(rt))
print("points with both", len(both))
cnt = Counter(); bym = defaultdict(Counter)
for k in both:
    p, q = cdf[k], rt[k]
    x, d1, d2 = k
    w = d1 * x / (d1 * x + d2); a, b = d1 / 2, d2 / 2
    side = "w<=mean" if w <= a / (a + b) else "w>mean"
    c1 = q == (0.5 - p) + 0.5
    c2 = p == (0.5 - q) + 0.5
    tag = ("q=1-p" if c1 else "") + ("|p=1-q" if c2 else "")
    if not tag: tag = "neither"
    cnt[tag] += 1; bym[side][tag] += 1
print(cnt)
for s in bym: print(s, dict(bym[s]))
# the 'neither' points
for k in both:
    p, q = cdf[k], rt[k]
    if q != (0.5 - p) + 0.5 and p != (0.5 - q) + 0.5:
        x, d1, d2 = k; w = d1 * x / (d1 * x + d2)
        print("neither", k, f"w={w:.4g} mean={d1/(d1+d2):.4g}", p, q)
print("--- exceptions to the mean rule")
for k in both:
    p, q = cdf[k], rt[k]
    x, d1, d2 = k
    w = d1 * x / (d1 * x + d2); a, b = d1 / 2, d2 / 2
    c1 = q == (0.5 - p) + 0.5; c2 = p == (0.5 - q) + 0.5
    if c1 and c2: continue
    lower = w <= a / (a + b)
    if (lower and not c1) or (not lower and not c2):
        print(k, f"w={w!r} mean={a/(a+b)!r}", "computed:", "p" if c1 else "q", p, q)
