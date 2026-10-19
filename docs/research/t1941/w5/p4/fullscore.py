"""Step (6): exact score of candidate sets on every [0.7,1.5) GAMMALN row (fresh extract, includes e9xx), by region."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import candidate, res
ROWS = sys.argv[1]; X = []; B = []; C = []
for l in open(ROWS):
    xh, bh, _, _, nm = l.split()
    if bh == 'nan': continue
    xv = float.fromhex(xh)
    if 0.7 <= xv < 1.5: X.append(xv); B.append(float.fromhex(bh)); C.append(nm)
X = np.array(X); B = np.array(B); C = np.array(C); X, ui = np.unique(X, return_index=True); B = B[ui]; C = C[ui]
e9 = np.char.startswith(C, 'kmyst-gam-e9')
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
sets = [('c5_f99t', (candidate('N3'), candidate('D3')))] + [(f, ld(os.path.join(HERE, f))) for f in sys.argv[2:]]
regions = [(0.7, 0.8), (0.8, 0.9), (0.9, 0.99), (0.99, 1.01), (1.01, 1.1), (1.1, 1.2), (1.2, 1.3), (1.3, 1.4), (1.4, 1.5)]
print('points %d (e9xx %d)' % (len(X), e9.sum()))
print('%-22s %7s %7s | ' % ('set', 'all', 'e9xx') + ' '.join('%.2f-%.2f' % r for r in regions))
for nm, (N, D) in sets:
    ok = res(X, N, D) == B
    print('%-22s %7.4f %7.4f | ' % (nm, ok.mean(), ok[e9].mean() if e9.any() else np.nan) + ' '.join('%9.3f' % ok[(X >= a) & (X < bb)].mean() for a, bb in regions))
