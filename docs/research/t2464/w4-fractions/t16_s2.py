import json, mpmath, numpy as np
from common4 import *
mpmath.mp.dps = 40; M = mpmath.mpf; U = 2.0 ** -53
out = json.load(open('probe/w4p1.out.json'))['sheets']; meta = json.load(open('probe/w4p1.meta.json'))
data = {(r['x'], r['a'], r['b']): r for r in load(lambda d: d['branch'] == 'direct' and d['src'] == 'w2')}
for S in ('S0', 'S2', 'S3'):
    pe = []; ce = []; me = []; qe = []
    for sh, i, xx, a, b, b0 in [m for m in meta if m[0] == S][::16]:
        c = out[sh]; P = c[f'B{i}']['v']; Q = c[f'C{i}']['v']; y0 = c[f'D{i}']['v']; r = data[(xx, a, b)]
        Y = M(y0); A, B = M(b0), M(a)
        Pex = Y ** (A - 1) * (1 - Y) ** (B - 1) / mpmath.beta(A, B)
        pe.append(float((M(P) - Pex) / Pex) / U)
        Qex = mpmath.betainc(M(a), M(b0), 0, M(xx), regularized=True); qe.append(float((M(Q) - Qex) / Qex) / U)
        ce.append(float((M(r['cdf']) - M(r['cdfx'])) / M(r['cdfx'])) / U)
    print(S, 'pdf piece err mean %+.2f sd %.2f | bpser piece err mean %+.2f | Excel cdf err mean %+.2f sd %.2f' % (np.mean(pe), np.std(pe), np.mean(qe), np.mean(ce), np.std(ce)))
