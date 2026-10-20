"""K1 fingerprint (GAMMA's G on [1,2): G = fl(N(y)/D(y)), raw-y (7,7) Horner, D0 = -1/16). No outer op, so the
target is Excel's G itself. Calibration: W4's AN2 simulated under each structure ("stored below k" as in
p5/maskmodel.py: ops at coefficient index j stored iff j < k; the top gsim stage k=0 is j = 6).
usage: k1fp.py K1ROWS.npz [NSUB]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..', 'p5'))
from fp import L, st, fit, stats
from maskmodel import chain
def ldND(path):
    N = D = None
    for l in open(path):
        if l.startswith('N '): N = np.array([float.fromhex(v) for v in l.split()[1:]])
        if l.startswith('D '): D = np.array([float.fromhex(v) for v in l.split()[1:]])
    return N, D
def G(x, N, D, kn=99, kd=99):
    xl = L(x); return st(chain(N, xl, kn)/chain(D, xl, kd))
z = np.load(sys.argv[1]); ns = int(sys.argv[2]) if len(sys.argv) > 2 else 300000
i = np.random.default_rng(1).choice(len(z['x']), min(ns, len(z['x'])), replace=False); x, y = z['x'][i], z['b'][i]
m = (x >= 1) & (x < 2); x, y = x[m], y[m]
a, b, r = fit(x, L(y), 7, 7, 1.0, 2.0)
print('K1 real (%d points): rms %.3f median %.3f' % ((len(x),) + stats(r)), flush=True)
N, D = ldND(os.path.join(HERE, '..', '..', 'g_candidate_w4_AN2.txt'))
print('  AN2 exact on these points (all stored): %.4f' % np.mean(G(x, N, D) == y))
for kn, kd, nm in ((99, 99, 'all stored'), (2, 3, 'W4 x87-top: N top5 / D top4 80-bit'), (3, 4, 'N top4 / D top3 80-bit'),
                   (1, 2, 'N top6 / D top5 80-bit'), (0, 0, 'all 80-bit')):
    ys = G(x, N, D, kn, kd); _, _, rs = fit(x, L(ys), 7, 7, 1.0, 2.0)
    print('  sim %-36s rms %.3f median %.3f' % ((nm,) + stats(rs)), flush=True)
