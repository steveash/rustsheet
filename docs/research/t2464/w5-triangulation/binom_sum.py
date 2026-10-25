"""Is BINOM.DIST's cdf a running sum of its own pmf?  cdf(k) vs cdf(k-1) + pmf(k) (Excel's own cells), and the
upper-tail analogue cdf(k) vs 1 - sum_{j>k} pmf(j); NEGBINOM likewise."""
import math, sys
from collections import defaultdict, Counter
import numpy as np
sys.path.insert(0, '.')
import w5lib as L
from w5lib import LD
for fam in (("BINOM.DIST", "BINOMDIST"), ("NEGBINOM.DIST", "NEGBINOMDIST")):
    pmf, cdf = {}, {}
    for c in L.cells(set(fam)):
        a = c["args"]
        if fam[0] == "BINOM.DIST":
            if len(a) < 4 or not isinstance(a[3], bool): continue
            k, n, p, cum = math.trunc(a[0]), math.trunc(a[1]), a[2], a[3]
        else:
            if len(a) < 4 or not isinstance(a[3], bool): continue
            k, n, p, cum = math.trunc(a[0]), math.trunc(a[1]), a[2], a[3]
        (cdf if cum else pmf)[(n, p, k)] = c["excel"]
    cnt = Counter()
    for (n, p, k), C in cdf.items():
        if k == 0 or (n, p, k - 1) not in cdf or (n, p, k) not in pmf: continue
        prev, f = cdf[(n, p, k - 1)], pmf[(n, p, k)]
        s_d = prev + f
        s_x = float(LD(prev) + LD(f))
        tag = ("dbl" if s_d == C else "") + ("|x87" if s_x == C else "")
        cnt[(("p<=.5" if p <= 0.5 else "p>.5"), tag or "no")] += 1
    print(fam[0], dict(sorted(cnt.items())))
    # forward full sum from k = 0 using Excel's pmfs
    cnt2 = Counter()
    keys = defaultdict(dict)
    for (n, p, k), f in pmf.items(): keys[(n, p)][k] = f
    for (n, p), d in keys.items():
        ks = sorted(d)
        if ks[0] != 0: continue
        s = 0.0; sx = LD(0)
        for k in ks:
            if k != ks[0] and k - 1 not in d: break
            s = s + d[k]; sx = sx + LD(d[k])
            if (n, p, k) in cdf:
                C = cdf[(n, p, k)]
                cnt2[("fwd-dbl", s == C)] += 1; cnt2[("fwd-ext", float(sx) == C)] += 1
    print("   running sum of Excel pmfs from 0:", dict(cnt2))
