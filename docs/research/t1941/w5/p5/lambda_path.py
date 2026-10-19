"""Partial-credit test (T-1941 W5 part 5): synthetic truth = base + m (function-preserving sloppy move, TSIZE ulps).
Evaluate base + lambda*m for lambda in [0,1]: exact-match rate and residual correlation corr(B-ref, sim-ref).
If neither rises before lambda ~ 1, no smooth objective exists along sloppy directions."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
exec(open(os.path.join(HERE, 'validate.py')).read().split('rows = []')[0])
from meter import L
idx = np.concatenate(segs); X = x[idx]
XL = X.astype(L); ref = (XL-1)*np.polyval(N0.astype(L)[::-1], XL)/np.polyval(D0.astype(L)[::-1], XL); U = np.abs(ref)*L(2.0**-52)
Bt = res(X, Nt, Dt); rB = ((Bt-ref)/U).astype(float); rB -= rB.mean()
def at(lam):
    N = N0.copy(); D = D0.copy()
    for i in range(len(N0)): N[i] += np.round(lam*_mv[i])*np.spacing(abs(N0[i]))
    for k, i in enumerate(range(1, len(D0))): D[i] += np.round(lam*_mv[len(N0)+k])*np.spacing(abs(D0[i]))
    return N, D
for lam in [0, 0.5, 0.9, 0.99, 0.999, 0.9999, 0.99999, 1.0]:
    N, D = at(lam); s = res(X, N, D); rs = ((s-ref)/U).astype(float); rs -= rs.mean()
    print('lambda %-8g exact %.4f  corr %.4f' % (lam, np.mean(s == Bt), float(rs@rB/np.sqrt((rs@rs)*(rB@rB)))))
