"""Regress the fraction residual (after lambda) on rounding errors of candidate x-dependent intermediates."""
import sys, mpmath
from common4 import *
import excelmath as em
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
rr = np.load('resid_%s.npy' % blk.replace(':', '_'))
xs = [r['x'] for r in rows]; ys = [1 - v for v in xs]
a, b = rows[0]['a'], rows[0]['b']
LNx = em.excel('LN', xs); LNy = em.excel('LN', ys)
def rel(vals, exact): return np.array([float((M(v) - e) / e) / U for v, e in zip(vals, exact)])
X = [M(v) for v in xs]
cols = {
 'LN x': rel(LNx, [mpmath.log(v) for v in X]),
 'LN y': rel(LNy, [mpmath.log(1 - v) for v in X]),
 'a*LNx': rel([x('*', a, l) for l in LNx], [a * mpmath.log(v) for v in X]),
 'b*LNy': rel([x('*', b, l) for l in LNy], [b * mpmath.log(1 - v) for v in X]),
 'x*y': rel([x('*', u, v) for u, v in zip(xs, ys)], [v * (1 - v) for v in X]),
 'y+1': rel([x('+', v, 1.0) for v in ys], [2 - v for v in X]),
 'lam+1': rel([x('+', x('-', a, x('*', a + b, u)), 1.0) for u in xs], [M(x('-', a, x('*', a + b, u))) + 1 for u in xs]),
 'aLNx+bLNy': rel([x('+', x('*', a, l1), x('*', b, l2)) for l1, l2 in zip(LNx, LNy)], [M(x('*', a, l1)) + M(x('*', b, l2)) for l1, l2 in zip(LNx, LNy)]),
}
for k, c in cols.items():
    print(f"{k:12s} sd {c.std():7.3f} corr {np.corrcoef(c, rr)[0,1]:+.3f}")
A = np.column_stack(list(cols.values()) + [np.ones(len(rr))])
coef, *_ = np.linalg.lstsq(A, rr, rcond=None)
print('joint:', ' '.join(f"{k}={c:+.3f}" for k, c in zip(cols, coef)), 'resid sd %.2f (from %.2f)' % (np.std(rr - A @ coef), rr.std()))
