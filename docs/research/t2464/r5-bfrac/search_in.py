"""Write search.c's input for a case/block: a b x y P cdf rx_hi rx_lo ee (hex)."""
import sys, mpmath
from exact_r import get
M = mpmath.mpf; U = M(2) ** -53
case, blk = (sys.argv[1], int(sys.argv[2])) if len(sys.argv) > 2 else ('t2464-w2f-xruns-1', 7)
rows = get(case, blk)
print(len(rows))
for r in rows:
    y = 1 - r['x']; hi = float(r['rx']); lo = float(r['rx'] - M(hi))
    ee = float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * y)) / r['rx'] - 1) / U)
    print(' '.join(v.hex() for v in (r['a'], r['b'], r['x'], y, r['pdf'], r['cdf'], hi, lo, ee)))
