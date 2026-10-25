"""What makes Excel's pdf noise (shared with the cdf) at a block: regress pdf rel error on rounding errors."""
import sys, mpmath
from common4 import *
import excelmath as em
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and not math.isnan(d['pdf']))
pe = np.array([float((M(r['pdf']) - M(r['pdfx'])) / M(r['pdfx'])) / U for r in rows])
xs = [r['x'] for r in rows]; ys = [1 - v for v in xs]; X = [M(v) for v in xs]
a, b = rows[0]['a'], rows[0]['b']
def rel(vals, exact): return np.array([float((M(v) - e) / e) / U for v, e in zip(vals, exact)])
def absu(vals, exact): return np.array([float(M(v) - e) / U for v, e in zip(vals, exact)])  # absolute error / 2^-53
LNx = em.excel('LN', xs); LNy = em.excel('LN', ys)
cols = {
 'tie y': rel(ys, [1 - v for v in X]),
 'lam_x rel': rel([x('-', a, x('*', a + b, u)) for u in xs], [M(a) - M(a + b) * v for v in X]),
 'lam_x(a-1,b-1)': rel([x('-', a - 1, x('*', a + b - 2, u)) for u in xs], [M(a - 1) - M(a + b - 2) * v for v in X]),
 'n*x abs': absu([x('*', a + b - 2, u) for u in xs], [M(a + b - 2) * v for v in X]),
 'n*y abs': absu([x('*', a + b - 2, v) for v in ys], [M(a + b - 2) * M(v) for v in ys]),
 'LNx abs': absu(LNx, [mpmath.log(v) for v in X]),
 'LNy abs': absu(LNy, [mpmath.log(M(v)) for v in ys]),
}
for k, c in cols.items():
    print(f"{k:16s} sd {c.std():8.3f} corr {np.corrcoef(c, pe)[0,1]:+.3f}")
A = np.column_stack(list(cols.values()) + [np.ones(len(pe))])
coef, *_ = np.linalg.lstsq(A, pe, rcond=None)
print('pdf sd %.2f; joint:' % pe.std(), ' '.join(f"{k}={c:+.3f}" for k, c in zip(cols, coef)), 'resid sd %.2f' % np.std(pe - A @ coef))
