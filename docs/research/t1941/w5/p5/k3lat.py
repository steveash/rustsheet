"""K3 near-null lattice (T-1941 W5 part 5; W4's nulllat.py adapted). Free coefficients: N3..N9, D3..D9 (the
plateau-read N0..N2, D1, D2 and D0 = 2^-13 fixed). J = change of A (in ulp(A)) per 1-ulp move of each coefficient
on a grid over [0.7, 1.5]; BKZ on [scale*I | lam*scale*J] gives short integer moves whose function change is small.
usage: k3lat.py SET LAM OUT.npy   (rows of OUT: integer moves in ulps, ordered N3..N9, D3..D9; prints fmax)"""
import numpy as np, mpmath as mp, math, sys, os
from fpylll import IntegerMatrix, BKZ
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
mp.mp.dps = 40
N0, D0 = ld(sys.argv[1]); lam = float(sys.argv[2])
idx = [('N', i) for i in range(3, 10)] + [('D', i) for i in range(3, 10)]
ul = np.array([math.ulp(abs((N0 if c == 'N' else D0)[i])) for c, i in idx])
J = []
for t in [mp.mpf(0.7)+mp.mpf(0.8)*k/80 for k in range(81)]:
    Nv = sum(mp.mpf(N0[i])*t**i for i in range(10)); Dv = sum(mp.mpf(D0[i])*t**i for i in range(10)); A = Nv/Dv
    u = mp.mpf(2)**(mp.floor(mp.log(abs(A), 2))-52)
    J.append([float(t**i/Dv/u) if c == 'N' else float(-A*t**i/Dv/u) for c, i in idx])
J = np.array(J)*ul
n, m, scale = len(idx), J.shape[0], 10**6
B = IntegerMatrix(n, n+m)
for j in range(n):
    B[j, j] = scale
    for r in range(m): B[j, n+r] = int(round(lam*J[r, j]*scale))
BKZ.reduction(B, BKZ.Param(block_size=min(12, n)))
V = np.array([[B[j, i]//scale for i in range(n)] for j in range(n)])
f = np.abs(J@V.T).max(0); o = np.argsort(f); V = V[o]; f = f[o]
for fv, v in zip(f, V):
    print('fmax %.4f  |v|max %9d  ' % (fv, np.abs(v).max()) + ' '.join('%s%d:%+d' % (c, i, d) for (c, i), d in zip(idx, v) if d))
np.save(sys.argv[3], V)
