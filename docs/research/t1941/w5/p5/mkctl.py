"""Planted controls for the K3 structure refit (T-1941 W5 part 5). Truth = ss_fc + a function-preserving move
(synth.make_truth: softest SVD directions, kmin 15, TSIZE 1e5, given seed); B synthesised under a mask
(maskmodel.py). Writes the truth set (hex, like ss_fc.txt) and, with a points npz, the control npz (x, b).
usage: mkctl.py NAME KN KD SEED [POINTS.npz OUT.npz]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from synth import make_truth
from maskmodel import res
name, kn, kd, seed = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
N0, D0 = ld(os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))
Nt, Dt, mv, fc = make_truth(N0, D0, seed=seed, tsize=1e5, kmin=15)
open(os.path.join(HERE, 'ctl_%s.txt' % name), 'w').write(' '.join(v.hex() for v in Nt)+'\n'+' '.join(v.hex() for v in Dt)+'\n')
print('%s: truth = ss_fc + move (ulps) %s; function change %.1e; mask N stored below %d, D below %d' % (name, mv.astype(int).tolist(), fc, kn, kd))
if len(sys.argv) > 6:
    z = np.load(sys.argv[5]); x = z['x']
    b = res(x, Nt, Dt, kn, kd); np.savez(sys.argv[6], x=x, b=b)
    print('  wrote %s: %d points; ss_fc all-stored exact on it %.4f, ss_fc under the same mask %.4f' % (
        sys.argv[6], len(x), np.mean(res(x[:400000], N0, D0) == b[:400000]), np.mean(res(x[:400000], N0, D0, kn, kd) == b[:400000])))
