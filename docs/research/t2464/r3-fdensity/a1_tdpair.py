"""a1: on r3-tdensity's paired columns (d1 = 1), is Excel's F.DIST(x,1,n,FALSE) its BETA.DIST(z,1/2,n/2,FALSE) at
z = fl(x/fl(n+x)) times a Jacobian?  Prints, per df and block, the spread of F's and H's errors (units 2^-53, against
the exact values at their own arguments) and of F/(H*J) - 1 with the exact Jacobian J = n/(n+x)^2."""
import collections
import statistics as st

import flib as F

mpf = F.mpf
rows = F.with_exact(F.tdens_rows())
g = collections.defaultdict(list)
for r in rows:
    x, n = r["x"], r["d2"]
    z = x / (n + x)
    H = r["beta_z"]
    if H is None:
        continue
    bex = F.beta_exact(z, 0.5, n / 2)
    J = mpf(n) / (mpf(n) + mpf(x)) ** 2
    # F against H at z: the Jacobian exactly at x, and the beta density's own error removed
    dF = float((mpf(r["excel"]) - r["exact"]) / r["exact"] * 2**53)
    dH = float((mpf(H) - bex) / bex * 2**53)
    rat = float((mpf(r["excel"]) / (mpf(H) * J) - 1) * 2**53)
    g[(n, r["blk"].split(",")[1] if "," in r["blk"] else r["blk"])].append((dF, dH, rat, r["excel"] == F.D(mpf(H) * J)))
print("df block n | F err mean sd | H err mean sd | F/(HJ)-1 mean sd | F == fl(H*Jexact)")
for k, v in sorted(g.items()):
    a = list(zip(*v))
    print(k, len(v), " | %.2f %.2f | %.2f %.2f | %.2f %.2f | %d" % (
        st.mean(a[0]), st.pstdev(a[0]), st.mean(a[1]), st.pstdev(a[1]), st.mean(a[2]), st.pstdev(a[2]), sum(a[3])))
