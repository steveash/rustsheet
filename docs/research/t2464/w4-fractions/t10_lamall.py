"""Per block: slope of Excel's direct-cdf rel error on the rel rounding error of lambda = fl(a - fl((a+b)x)), compared
with bfrac's exact sensitivity S = dln r/dln lambda (finite difference in mpmath)."""
import mpmath, collections
from common4 import *
from ideal_c import bfrac
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
rows = load(lambda d: d['src'] == 'w2' and d['branch'] == 'direct')
g = collections.defaultdict(list)
for r in rows: g[r['blk']].append(r)
for blk, rs in sorted(g.items()):
    a, b = rs[0]['a'], rs[0]['b']
    if 'absweep' in blk: continue
    ce = np.array([float((M(r['cdf']) - M(r['cdfx'])) / M(r['cdfx'])) / U for r in rs])
    dl = []
    for r in rs:
        X = M(r['x']); lt = M(a) - M(a + b) * X
        dl.append(float((M(x('-', a, x('*', a + b, r['x']))) - lt) / lt) / U)
    dl = np.array(dl)
    A = np.column_stack([dl, np.ones(len(dl))]); co, *_ = np.linalg.lstsq(A, ce, rcond=None)
    res = ce - A @ co; se = res.std() / (dl.std() * math.sqrt(len(dl)))
    xm = M(rs[len(rs) // 2]['x']); A_, B_ = M(a), M(b); lam = A_ - (A_ + B_) * xm; h = M(10) ** -20
    S = float((mpmath.log(bfrac(A_, B_, xm, 1 - xm, lam * (1 + h))) - mpmath.log(bfrac(A_, B_, xm, 1 - xm, lam))) / h) if a > 1 and b > 1 else float('nan')
    print(f"{blk:10s} a={a:<6g} b={b:<6g} x~{float(xm):.4f} n={len(rs)} lam={float(lam):+.3f} sd(dlam)={dl.std():6.2f} slope={co[0]:+.3f}({se:.3f}) bfrac S={S:+.3f}")
