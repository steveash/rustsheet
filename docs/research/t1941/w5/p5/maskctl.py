"""Mask-gain control (T-1941 W5 part 5): does dropping stores (80-bit head) raise the exact rate of ss_fc also on a
FULLY STORED synthetic truth? If yes, a mask gain without refit is not evidence of an unstored head (removing a
mis-phased rounding beats keeping it). usage: maskctl.py TRUTHSET   (B from TRUTHSET, all stored)"""
import numpy as np, sys; sys.path.insert(0, '.')
from hypolib import ld
from meter import L, st
N, D = ld('../p4/ss_fc.txt'); Nt, Dt = ld(sys.argv[1])
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz')
i = np.random.default_rng(7).choice(len(z['x']), 400000, replace=False); x = z['x'][i]; xl = L(x); t = L(st(xl-L(1.0)))
def chain(C, nf):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = L(st(p)) if j < nf else p
        s = p+L(C[j]); s = L(st(s)) if j < nf else s
    return s
b = st(t*L(st(chain(Nt, 99)/chain(Dt, 99))))
def ex(nf, df): return np.mean(st(t*L(st(chain(N, nf)/chain(D, df)))) == b)
print('truth %s (all stored); ss_fc scored with masks' % sys.argv[1])
for nf in (99, 5, 3, 2, 1):
    print('N stored below %2d: ' % nf + ' '.join('D<%d:%.4f' % (df, ex(nf, df)) for df in (99, 3, 2, 1, 0)))
