"""Exact-rate tolerance of single-coefficient moves on a planted K3 truth (T-2122 run 5, HANDOFF2 17): 128 runs of
2,048 consecutive doubles in [0.7, 1.49), B synthesised from TRUTHSET, then the truth with one N coefficient moved
by u ulps is scored with the exact x87 scorer. Run from w5/p5. usage: k3_tol.py TRUTHSET"""
import sys, math, numpy as np
sys.path.insert(0, '.'); sys.path.insert(0, '..')
from hypolib import ld
from maskmodel import res
from sc import score
Nt, Dt = ld(sys.argv[1])
rng = np.random.default_rng(0); xs = []
for a in rng.uniform(0.7, 1.49, 128):
    v = [a]
    for _ in range(2047): v.append(math.nextafter(v[-1], 2))
    xs += v
x = np.ascontiguousarray(np.array(xs)); b = np.ascontiguousarray(np.asarray(res(x, Nt, Dt), dtype=np.float64))
def sc(k, u):
    N = Nt.copy(); N[k] += u*np.spacing(abs(N[k])); return score(x, b, N, Dt)
print('truth', score(x, b, Nt, Dt))
for k, us in ((9, (389, 1e4, 1e5, 1e6, 4e6)), (8, (1, 2, 3, 10, 100, 3e3)), (7, (1, 3, 10, 100)), (5, (1, 10, 100, 1e3)), (4, (1, 10, 100))):
    print('N%d' % k, ' '.join('%+g:%.4f' % (u, sc(k, u)) for u in us), flush=True)
