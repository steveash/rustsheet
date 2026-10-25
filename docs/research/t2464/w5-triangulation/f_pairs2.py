"""Test the bratio dispatch's directly computed tail against the F pairs (which tail is the complement)."""
import math, sys
from collections import Counter, defaultdict
sys.path.insert(0, '.')
import w5lib as L
from dispatch import bratio_branch
cdf, rt = {}, {}
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a = c["args"]
    if c["fn"] == "F.DIST" and a[3] is not True: continue
    key = (a[0], math.trunc(a[1]), math.trunc(a[2]))
    (cdf if c["fn"] == "F.DIST" else rt)[key] = c["excel"]
both = sorted(set(cdf) & set(rt))
res = defaultdict(Counter); res_mean = Counter()
for k in both:
    p, q = cdf[k], rt[k]
    x, d1, d2 = k
    if not x > 0: continue
    s = d1 * x + d2; w = d1 * x / s; v = d2 / s
    a, b = d1 / 2, d2 / 2
    c1 = q == (0.5 - p) + 0.5; c2 = p == (0.5 - q) + 0.5
    obs = "both" if (c1 and c2) else ("lower" if c1 else "upper")
    br, ind, pred = bratio_branch(a, b, w, v)
    res[br][obs + ("=ok" if obs in ("both", pred) else "=BAD")] += 1
    m = "lower" if w <= a / (a + b) else "upper"
    res_mean[obs + ("=ok" if obs in ("both", m) else "=BAD")] += 1
    if obs not in ("both", pred):
        print("BAD", k, br, ind, pred, f"w={w:.5g}")
for br in sorted(res): print(f"{br:22s}", dict(res[br]))
print("mean-switch rule:", dict(res_mean))
