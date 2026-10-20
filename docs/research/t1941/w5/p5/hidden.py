"""Hidden-term test on K3 (T-1941 W5 part 5; the coordinator's K2 lead). At stage j of a chain, the stored sum is
S_j = fl(P_j + C_j + c/x) (formed in extended, one store) and the next sum down uses C_{j-1} - c:
S_{j-1} = fl(P_{j-1} + C_{j-1} - c). Algebraically a no-op; it changes only the roundings. Scan c in units of
ulp(S_{j-1}) and score exact on real data. usage: hidden.py SET   env: NSUB (50000), KR (10000), STEP (16), STAGES, FINE (c unit = ulp(S_j)/FINE)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1])
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz'))
i = np.random.default_rng(5).choice(len(z['x']), int(os.environ.get('NSUB', 50000)), replace=False); x, b = z['x'][i], z['b'][i]
xl = L(x); t = L(st(xl-L(1.0)))
def chain(C, j=None, c=0.0, upto=None):
    s = L(C[-1])
    for k in range(len(C)-2, -1, -1):
        p = L(st(s*xl)); v = p+L(C[k])
        if j is not None and k == j: v = v+L(c)/xl
        if j is not None and k == j-1: v = v-L(c)
        s = L(st(v))
    return s
nb, db = chain(N), chain(D)
base = np.mean(st(t*L(st(nb/db))) == b)
KR, ST = int(float(os.environ.get('KR', 10000))), int(os.environ.get('STEP', 16))
stages = os.environ.get('STAGES', ','.join('%s%d' % (ch, j) for ch in 'ND' for j in range(9, 0, -1))).split(',')
print('base exact %.4f on %d points' % (base, len(x)), flush=True)
for sg in stages:
    ch, j = sg[0], int(sg[1:]); C = N if ch == 'N' else D
    if j >= len(C) or j < 1: continue
    u = float(np.median(np.spacing(np.abs(chain(C)[:1000].astype(float)))))  # rough scale; refine: ulp of S_{j-1}
    # ulp of S_{j-1} at the median point
    s = L(C[-1]); vals = []
    for k in range(len(C)-2, j-2 if not os.environ.get('FINE') else j-1, -1):
        s = L(st(L(st(s*xl))+L(C[k])))
    u = float(np.median(np.spacing(np.abs(s.astype(float)))))   # ulp(S_{j-1}); FINE: ulp(S_j)/FINE
    if os.environ.get('FINE'): u /= int(os.environ['FINE'])
    ks = np.arange(-KR, KR+1, ST); sc = []
    for k in ks:
        cn = chain(C, j, k*u)
        y = st(t*L(st(cn/db))) if ch == 'N' else st(t*L(st(nb/cn)))
        sc.append(np.mean(y == b))
    sc = np.array(sc); o = np.argsort(sc)[::-1]
    print('%s.S%d (u %.3g): best %s; at c=0 %.4f; median %.4f' % (ch, j, u, ' '.join('%+d:%.4f' % (ks[q], sc[q]) for q in o[:4]), sc[ks == 0][0], np.median(sc)), flush=True)
