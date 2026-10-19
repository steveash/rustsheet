"""Degree-reduction test (T-1941 W5 part 3): relative minimax of c5_f99t's own function at degree (m,n),
D0 normalised to 2^-13, rounded to doubles, scored exactly. Saves sets to p3/deg_m_n.txt (ascending hex)."""
import mpmath as mp, numpy as np, sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from remez2 import remez2
from sc import candidate, score
from metrics import XF, BF, XN, BN
mp.mp.dps = 40
Nc = [mp.mpf(v) for v in candidate('N3')[::-1]]; Dc = [mp.mpf(v) for v in candidate('D3')[::-1]]
g = lambda x: mp.polyval(Nc, x)/mp.polyval(Dc, x)
print('c5 (9,9): far %.4f near %.4f' % (score(XF, BF, candidate('N3'), candidate('D3')), score(XN, BN, candidate('N3'), candidate('D3'))))
for m, n in [(9, 8), (8, 8), (8, 9), (9, 9), (8, 7), (7, 8)]:
    p, q, E, X = remez2(g, 0.7, 1.5, m, n, dps=40, grid=500, tol='1e-12')
    s = mp.mpf(2)**-13
    N = np.array([float(v*s) for v in p]); D = np.array([float(v*s) for v in q])
    open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'deg_%d_%d.txt' % (m, n)), 'w').write(' '.join(v.hex() for v in N) + '\n' + ' '.join(v.hex() for v in D) + '\n')
    print('(%d,%d): E %.1e  far %.4f near %.4f  top N %.2e D %.2e' % (m, n, float(E), score(XF, BF, N, D), score(XN, BN, N, D), N[-1], D[-1]), flush=True)
