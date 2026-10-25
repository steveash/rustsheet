"""F pairs (cdf, rt at one point) and BINOM/NEGBINOM pairs: which tail is computed, at closed-form shapes, against
the closed-form rule (b0 = 1: oriented lower; a0 = 1, y0 <= 0.9: oriented upper) and against bratio's dispatch."""
import math, sys
from collections import Counter
sys.path.insert(0, '.')
import w5lib as L
from dispatch import bratio_branch
from build_evidence import closed_kind
cdf, rt = {}, {}
for c in L.cells({"F.DIST", "F.DIST.RT", "FDIST"}):
    a = c["args"]
    if c["fn"] == "F.DIST" and a[3] is not True: continue
    key = (a[0], math.trunc(a[1]), math.trunc(a[2]))
    (cdf if c["fn"] == "F.DIST" else rt)[key] = c["excel"]
cnt = Counter()
for k in set(cdf) & set(rt):
    X, d1, d2 = k
    if not X > 0: continue
    p, q = cdf[k], rt[k]
    c1 = q == (0.5 - p) + 0.5; c2 = p == (0.5 - q) + 0.5
    if c1 == c2: continue
    obs = "lower" if c1 else "upper"
    a, b = d1 / 2, d2 / 2
    P = d1 * X; w = 1 / (1 + d2 / P); v = 1 / (1 + P / d2)
    ck, comp = closed_kind(a, b, w, v)
    if ck is None: continue
    br, ind, bcomp = bratio_branch(a, b, w, v)
    cnt[(ck, "closed-rule " + ("ok" if comp == obs else "BAD"), "bratio " + ("ok" if bcomp == obs else "BAD"))] += 1
for k in sorted(cnt): print(k, cnt[k])
