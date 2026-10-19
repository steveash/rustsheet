import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
exec(open(os.path.join(HERE, 'validate.py')).read().split('rows = []')[0])
from meter import sites_and_weights, L, st, PH
s = segs[3]; xx = x[s]
W, _ = sites_and_weights(N0, D0, xx); Wt, _ = sites_and_weights(Nt, Dt, xx)
for k in [('N', 'S', 7), ('N', 'S', 6), ('D', 'S', 5)]:
    pre, sv, w = W[k]; pre_t, sv_t, _ = Wt[k]
    u = np.spacing(np.abs(sv)).astype(L); ut = np.spacing(np.abs(sv_t)).astype(L)
    et = ((L(sv_t)-pre_t)/ut).astype(float)          # truth's rounding error in its own ulps
    dphi = ((pre_t-pre)/u).astype(float)
    print(k, 'u==ut', bool(np.all(u == ut)), 'dphi spread (min,max of frac) %.3f %.3f' % ((dphi % 1).min(), (dphi % 1).max()), 'dphi range %.3g..%.3g' % (dphi.min(), dphi.max()))
    best = []
    for ph in PH:
        v = pre+L(ph)*u; e = ((L(st(v))-v)/u).astype(float); best.append(np.mean(np.abs(e-et) < 1e-9))
    i = int(np.argmax(best)); print('    best phi %.3f match %.3f | median dphi mod 1 %.3f' % (PH[i], best[i], np.median(dphi % 1)))
