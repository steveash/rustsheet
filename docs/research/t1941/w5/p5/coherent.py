"""Coherent multi-run detectability of top-site lines (T-1941 W5 part 5).
For each top site, per-run correlation curves C_r(phi) between the residual (synthetic truth B - ref) and the
candidate's template at phase phi. Z = (sum_r C_r(true phi_r) - mean) / sd, where the null distribution comes
from random per-run phases. Z >> 5 means a global fit of the site's smooth phase function is statistically
feasible with this much data."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
exec(open(os.path.join(HERE, 'validate.py')).read().split('rows = []')[0])
from meter import sites_and_weights, template, resid, L, PH
SITES = [('N', 'S', 7), ('N', 'P', 6), ('N', 'S', 6), ('N', 'P', 5), ('D', 'S', 6), ('D', 'P', 5), ('D', 'S', 5), ('D', 'P', 4)]
acc = {k: ([], []) for k in SITES}
for s in segs:
    xx = x[s]; tgt = res(xx, Nt, Dt)
    W, ref = sites_and_weights(N0, D0, xx); Wt, _ = sites_and_weights(Nt, Dt, xx); r = resid(xx, tgt, ref)
    for k in SITES:
        pre, sv, w = W[k]; T = template(pre, sv, w); Tc = T-T.mean(1, keepdims=True)
        C = (Tc@r)/np.maximum(np.sqrt((Tc*Tc).sum(1)), 1e-30)
        u = np.spacing(np.abs(sv)).astype(L); pt = float(np.median(((Wt[k][0]-pre)/u).astype(float)) % 1.0)
        acc[k][0].append(C); acc[k][1].append(int(round(pt*len(PH))) % len(PH))
rng = np.random.default_rng(0)
for k in SITES:
    C = np.array(acc[k][0]); ti = np.array(acc[k][1]); R = len(ti)
    true = C[np.arange(R), ti].sum()
    null = np.array([C[np.arange(R), rng.integers(0, len(PH), R)].sum() for _ in range(2000)])
    print('%s.%s%d runs %d  Z(true) = %+.1f   (best per-run-max sum Z %+.1f)' % (k[0], k[1], k[2], R, (true-null.mean())/null.std(), (C.max(1).sum()-null.mean())/null.std()))
