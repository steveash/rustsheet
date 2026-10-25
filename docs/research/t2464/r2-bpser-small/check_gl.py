"""Excel's GAMMALN (probe t2464-r2-bpser-small-gl) against the engine model at the shapes' a, b, a+b."""
from collections import Counter

import lib

got = lib.probed_gl()
vals = sorted(got)
model = lib.bx.em.excel("GAMMALN", vals)
c = Counter()
for v, m in zip(vals, model):
    band = "v<1" if v < 1 else "1<=v<2" if v < 2 else "2<=v<8" if v < 8 else "v>=8"
    c[(band, "n")] += 1
    c[(band, "model exact")] += m == got[v]
    c[(band, "model within 1 ulp")] += abs(m - got[v]) <= abs(got[v]) * 2.3e-16
for k in sorted(c):
    print(k, c[k])
