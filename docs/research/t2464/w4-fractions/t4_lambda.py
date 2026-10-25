"""Regress Excel's implied fraction error q (block where cdf shares pdf) on the relative rounding error of candidate
lambda computations, times the fraction's exact sensitivity S = dln r/dln lambda (computed numerically)."""
import sys, mpmath
from common4 import *
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
q = []
for r in rows:
    X, A = M(r['x']), M(r['a'])
    rx = M(r['cdfx']) / (M(r['pdfx']) * X * (1 - X))
    q.append(float(M(r['cdf']) / (M(r['pdf']) * X * M(1 - r['x'])) / rx - 1) / U)
q = np.array(q)
a, b = rows[0]['a'], rows[0]['b']
cands = {
  'lam=fl(a-fl((a+b)x))': lambda xx: float(LD(a) - LD(x('*', a + b, xx))),
  'lam=a-(a+b)x ext->dbl': lambda xx: float(LD(a) - LD(a + b) * LD(xx)),
  'lam=fl(a-fl(a*x+b*x))': lambda xx: x('-', a, x('+', x('*', a, xx), x('*', b, xx))),
  'lam=fl(fl((a+b)y)-b)': lambda xx: x('-', x('*', a + b, 1 - xx), b),
  'lam=fl(a*y-b*x)': lambda xx: x('-', x('*', a, 1 - xx), x('*', b, xx)),
}
cols = {}
for nm, f in cands.items():
    d = []
    for r in rows:
        X = M(r['x']); lt = M(a) - M(a + b) * X
        d.append(float((M(f(r['x'])) - lt) / lt) / U)
    cols[nm] = np.array(d)
for nm, c in cols.items():
    if c.std() == 0: print(nm, 'const'); continue
    A_ = np.column_stack([c, np.ones(len(c))]); coef, *_ = np.linalg.lstsq(A_, q, rcond=None)
    print(f"{nm:28s} sd(dlam) {c.std():5.2f} slope {coef[0]:+.3f} corr {np.corrcoef(c, q)[0,1]:+.3f} resid sd {np.std(q - A_ @ coef):.2f}")

# decompose bfrac double simulations: e = S*dlam + internal
names = [f"o{o}l{l}e{e}" for o in (0, 1) for l in (0, 1, 2, 3) for e in ('1.5e-14', '1e-15', '2.2e-16', '1e-14', '3e-15')]
dl = cols['lam=fl(a-fl((a+b)x))']
for mode in (0, 1):
    res = run_cf(rows, mode, exe='c/bf%d.bin')
    for ci in (0, 15):
        e = []
        for r, h in zip(rows, res):
            X = M(r['x']); rx = M(r['cdfx']) / (M(r['pdfx']) * X * (1 - X))
            e.append(float(M(str(h[ci])) * M(a) / (rx * M(a)) - 1) / U if False else float(M(str(h[ci])) / rx - 1) / U)
        e = np.array(e)
        A_ = np.column_stack([dl, np.ones(len(dl))]); ce, *_ = np.linalg.lstsq(A_, e, rcond=None)
        inner = e - A_ @ ce
        qa = q - np.linalg.lstsq(A_, q, rcond=None)[0] @ A_.T
        print(f"mode {mode} {names[ci]}: model slope on dlam {ce[0]:+.3f}, model internal sd {inner.std():.2f}; corr(Excel resid, model internal) {np.corrcoef(qa, inner)[0,1]:+.3f}")
