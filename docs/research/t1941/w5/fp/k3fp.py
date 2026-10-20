"""K3 fingerprint check (should reproduce W2: real rms 1.013 / median 0.663, all-stored 1.018 / 0.665).
A = fl(N/D) on [0.7,1.5), y = fl(fl(x-1) A). usage: k3fp.py ROWS.npz [NSUB]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..', 'p5'))
from fp import L, st, target, fit, stats, to_monomial
from maskmodel import res as k3res
z = np.load(sys.argv[1]); ns = int(sys.argv[2]) if len(sys.argv) > 2 else 400000
i = np.random.default_rng(1).choice(len(z['x']), ns, replace=False); x, y = z['x'][i], z['b'][i]
m = np.abs(x-1) > 1e-2; x, y = x[m], y[m]
t = st(L(x)-L(1.0))
outer = lambda A: st(L(t)*L(A))
A = target(y, outer, st(L(y)/L(t)))
ok = np.isfinite(A); print('targets found for %.4f of points' % ok.mean())
a, bb, r = fit(x[ok], A[ok], 9, 9, 0.7, 1.5)
print('real: rms %.3f median %.3f' % stats(r), flush=True)
from hypolib import ld
Nd, Dd = ld(os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))   # calibration function: ss_fc (D(0) of a free (9,9) LS rational is undetermined)
for kn, kd, nm in ((99, 99, 'all stored'), (3, 3, 'stored below 3 both'), (2, 0, 'N<2, D 80-bit'), (0, 0, 'all 80-bit')):
    ys = k3res(x, Nd, Dd, kn, kd); As = target(ys, outer, st(L(ys)/L(t))); okk = np.isfinite(As)
    _, _, rs = fit(x[okk], As[okk], 9, 9, 0.7, 1.5)
    print('sim %-20s rms %.3f median %.3f' % ((nm,) + stats(rs)), flush=True)
