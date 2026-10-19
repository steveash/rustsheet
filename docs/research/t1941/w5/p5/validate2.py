"""Per-site validation of the continuous-template meter on a function-preserving synthetic move (see validate.py)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
exec(open(os.path.join(HERE, 'validate.py')).read().split('rows = []')[0])
from meter import sites_and_weights, template, resid, site_phase, L
TT = {}
for s in segs:
    xx = x[s]; tgt = res(xx, Nt, Dt); xm = xx[len(xx)//2]
    W, ref = sites_and_weights(N0, D0, xx)
    Wt, _ = sites_and_weights(Nt, Dt, xx)
    r = resid(xx, tgt, ref)
    for key, (pre, sv, w) in W.items():
        ph, ctr, amp = site_phase(r, template(pre, sv, w))
        # truth phase: (pre_true - pre_cand)/u mean over run (smooth part)
        pre_t = Wt[key][0]; u = np.spacing(np.abs(sv)).astype(L)
        pt = float(np.median(((pre_t-pre)/u).astype(float)) % 1.0)
        TT.setdefault(key, []).append((ph, pt, ctr, amp))
for key in sorted(TT, key=lambda k: (k[0], -k[2], k[1])):
    a = np.array(TT[key]); dd = (a[:, 0]-a[:, 1]+0.5) % 1-0.5; vis = a[:, 2] > 0.1
    print('%s.%s%d  amp %.3f  contrast %.2f  visible %.2f  phase rms(vis) %s' % (key[0], key[1], key[2], np.median(a[:, 3]), np.median(a[:, 2]), vis.mean(), ('%.3f' % np.sqrt(np.mean(dd[vis]**2))) if vis.any() else '-'))
