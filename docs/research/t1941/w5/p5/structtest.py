"""Model-structure test on real data (T-1941 W5 part 5): drop the fl53 store at one stage at a time (keep the
fl64 value) and measure the exact-match rate. A stage whose store is not in Excel's code should raise it sharply.
usage: structtest.py SET [ROWSNPZ]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from hypolib import ld
from meter import L, st
from sc import res
N, D = ld(sys.argv[1])
z = np.load(sys.argv[2] if len(sys.argv) > 2 else '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
rng = np.random.default_rng(1); i = rng.choice(len(z['x']), 300000, replace=False); x = z['x'][i]; b = z['b'][i]
def chain(C, xl, skip):
    s = L(C[-1]); 
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = p if ('P', j) in skip else L(st(p))
        s = p+L(C[j]); s = s if ('S', j) in skip else L(st(s))
    return s
def model(skip=()):
    xl = L(x); n = chain(N, xl, {k[1:] for k in skip if k[0] == 'N'}); d = chain(D, xl, {k[1:] for k in skip if k[0] == 'D'})
    a = n/d; a = a if 'A' in skip else L(st(a))
    t = L(st(xl-L(1.0)))
    return st(t*a)
base = model(); ref = res(x, N, D)
print('python model == libsc: %.6f; exact base %.4f' % (np.mean(base == ref), np.mean(base == b)), flush=True)
out = []
for ch in 'ND':
    for kind in 'PS':
        for j in range(9, -1, -1):
            if kind == 'P' and j > 8: continue
            if kind == 'S' and j > 8: continue
            r = np.mean(model({(ch, kind, j)}) == b); out.append((r, '%s.%s%d' % (ch, kind, j)))
out.append((np.mean(model({'A'}) == b), 'A'))
out.sort(reverse=True)
print(' '.join('%s %.4f' % (k, r) for r, k in out))
