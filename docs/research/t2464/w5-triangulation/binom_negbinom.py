"""Same incomplete beta through two functions: NEGBINOM.DIST(f,r,p,TRUE) = I_p(r, f+1) = 1 - BINOM.DIST(r-1, f+r, p, TRUE).
If both are one kernel evaluation, one equals (0.5 - other) + 0.5 or 1 - other to the bit."""
import math, sys
from collections import Counter
sys.path.insert(0, '.')
import w5lib as L
B, N = {}, {}
for c in L.cells({"BINOM.DIST", "BINOMDIST"}):
    a = c["args"]
    if len(a) < 4 or not isinstance(a[3], bool) or not a[3]: continue
    B[(math.trunc(a[0]), math.trunc(a[1]), a[2])] = c["excel"]
for c in L.cells({"NEGBINOM.DIST", "NEGBINOMDIST"}):
    a = c["args"]
    if len(a) < 4 or not isinstance(a[3], bool) or not a[3]: continue
    N[(math.trunc(a[0]), math.trunc(a[1]), a[2])] = c["excel"]
cnt = Counter(); ex = []
for (f, r, p), v in N.items():
    k = (r - 1, f + r, p)
    if k in B:
        b = B[k]
        tag = ("eq(0.5-b+0.5)" if v == (0.5 - b) + 0.5 else "") + ("|b=(0.5-v+0.5)" if b == (0.5 - v) + 0.5 else "")
        cnt[tag or "no"] += 1
        if not tag: ex.append(((f, r, p), v, b, (v - (1 - b)) / math.ulp(v)))
print(dict(cnt))
for e in ex[:15]: print(e)
from dispatch import bratio_branch
from collections import defaultdict
res = defaultdict(Counter); bad = []
for (f, r, p), v in N.items():
    k = (r - 1, f + r, p)
    if k not in B: continue
    b = B[k]
    c1 = v == (0.5 - b) + 0.5   # NEGBINOM is the complement: BINOM (upper of I_p(r,f+1)) computed
    c2 = b == (0.5 - v) + 0.5   # BINOM is the complement: NEGBINOM (lower) computed
    if c1 == c2: continue
    obs = "upper" if c1 else "lower"
    br, ind, pred = bratio_branch(float(r), float(f + 1), p, 1 - p)
    res[br][obs + ("=ok" if obs == pred else "=BAD")] += 1
    if obs != pred: bad.append(((f, r, p), br, ind, pred))
for br in sorted(res): print(f"{br:22s}", dict(res[br]))
for x in bad[:20]: print("BAD", x)
