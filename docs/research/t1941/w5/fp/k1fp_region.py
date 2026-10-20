"""K1 regional fingerprint (W4's request): real vs planted plain1 (all stored), LS (7,7) Chebyshev fit per region.
usage: k1fp_region.py K1ROWS.npz"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from fp import L, fit, stats
from k1fp_lib import ldND, G
z = np.load(sys.argv[1]); X, Y = z['x'], z['b']
N, D = ldND(os.path.join(HERE, 'plain1.txt'))
for lo, hi in ((1.2, 1.7), (1.84, 1.97), (1.0, 1.2), (1.7, 1.84), (1.97, 2.0)):
    m = np.where((X >= lo) & (X < hi))[0]
    for seed in (1, 2, 3):
        i = np.random.default_rng(seed).choice(m, min(150000, len(m)), replace=False); x, y = X[i], Y[i]
        _, _, r = fit(x, L(y), 7, 7, lo, hi); ys = G(x, N, D); _, _, rs = fit(x, L(ys), 7, 7, lo, hi)
        rr, rp = r[np.abs(r) < 20], rs[np.abs(rs) < 20]
        print('[%.2f,%.2f) seed %d (%6d pts): real rms %.3f med %.3f | planted plain1 rms %.3f med %.3f | real-planted %+.3f | exact(plain1) %.3f | outliers %d' % (
            lo, hi, seed, len(x), *stats(rr), *stats(rp), stats(rr)[0]-stats(rp)[0], np.mean(ys == y), (np.abs(r) >= 20).sum()), flush=True)
