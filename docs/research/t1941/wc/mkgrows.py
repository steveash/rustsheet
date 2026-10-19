# build grows.npz (y, Excel G, Gamma hi/lo split) from review/g_rows.txt; ~1 min with mpmath
import numpy as np, mpmath as mp, sys

y = []
g = []
for l in open(sys.argv[1] if len(sys.argv) > 1 else "g_rows.txt"):
    p = l.split()
    y.append(float.fromhex(p[0]))
    g.append(float.fromhex(p[1]))
y = np.array(y)
g = np.array(g)
mp.mp.dps = 25
hi = np.empty(len(y))
lo = np.empty(len(y))
for i, v in enumerate(y):
    G_ = mp.gamma(mp.mpf(v))
    h = float(G_)
    hi[i] = h
    lo[i] = float((G_ - mp.mpf(h)) / mp.mpf(np.spacing(h)))
np.savez("grows.npz", y=y, g=g, gam=hi, gamlo=lo)
