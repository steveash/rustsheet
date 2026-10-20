"""Common-scale test (T-1941 W5 part 5; W6's D0 lesson): (N, D) -> (fl(c N), fl(c D)) keeps the function exactly and
changes only the stage roundings. If Excel's D0 is 2^-13 * c* with c* != 1, the exact rate should peak at c*.
c = 1 + k 2^-52 (exact rational products, rounded to nearest). usage: cscan.py SET   env: KR, STEP, NSUB, NPZ"""
import numpy as np, sys, os
from fractions import Fraction as Fr
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from hypolib import ld
from sc import score_batch
N, D = ld(sys.argv[1])
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz'))
i = np.random.default_rng(0).choice(len(z['x']), int(os.environ.get('NSUB', 100000)), replace=False); x, b = z['x'][i], z['b'][i]
KR = int(float(os.environ.get('KR', 8192))); ST = int(os.environ.get('STEP', 1))
ks = np.arange(-KR, KR+1, ST)
def scaled(C, k):
    c = Fr(1) + Fr(int(k), 2**52)
    return np.array([float(Fr(float(v))*c) for v in C])   # float(Fraction) rounds to nearest
P = np.array([scaled(N, k) for k in ks]); Q = np.array([scaled(D, k) for k in ks])
s = score_batch(x, b, P, Q); o = np.argsort(s)[::-1]
print('k=0: %.4f; median %.4f, sd %.4f; top: %s' % (s[ks == 0][0], np.median(s), s.std(), ' '.join('%+d:%.4f' % (ks[j], s[j]) for j in o[:8])))
np.save('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/cscan.npy', np.c_[ks, s])
