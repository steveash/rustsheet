"""T.DIST(t,n,TRUE), T.DIST.RT, T.DIST.2T, TDIST at the same (t, n): how they derive from one tail L."""
import math, sys
from collections import Counter, defaultdict
sys.path.insert(0, '.')
import w5lib as L
V = defaultdict(dict)
for c in L.cells({"T.DIST", "T.DIST.RT", "T.DIST.2T", "TDIST"}):
    a = c["args"]
    if c["fn"] == "T.DIST":
        if a[-1] is not True: continue
        t, n = a[0], math.trunc(a[1]); key = "cdf"
    elif c["fn"] == "TDIST":
        t, n = a[0], math.trunc(a[1]); key = f"TDIST{math.trunc(a[2])}"
    else:
        t, n = a[0], math.trunc(a[1]); key = c["fn"]
    V[(t, n)][key] = c["excel"]
cnt = Counter()
for (t, n), d in V.items():
    if "T.DIST.RT" in d:
        Lr = d["T.DIST.RT"]
        if "T.DIST.2T" in d: cnt[("2T == 2*RT", d["T.DIST.2T"] == 2 * Lr)] += 1
        if "TDIST1" in d: cnt[("TDIST1 == RT", d["TDIST1"] == Lr)] += 1
        if "TDIST2" in d: cnt[("TDIST2 == 2RT", d["TDIST2"] == 2 * Lr)] += 1
        if "cdf" in d and t > 0:
            C = d["cdf"]
            cnt[(f"cdf(t>0,{'t<=1' if t <= 1 else 't>1'})", "1-L" if C == 1 - Lr else "0.5-L+0.5" if C == (0.5 - Lr) + 0.5 else "no")] += 1
    if (-t, n) in V and "cdf" in V[(-t, n)] and t > 0:
        if "T.DIST.RT" in d: cnt[("cdf(-t) == RT(t)", V[(-t, n)]["cdf"] == d["T.DIST.RT"])] += 1
        if "cdf" in d:
            a, b = d["cdf"], V[(-t, n)]["cdf"]
            cnt[("cdf(t) vs cdf(-t)", "1-b" if a == 1 - b else "no")] += 1
for k in sorted(cnt, key=str): print(k, cnt[k])
