"""Residual of the current model per run: Excel's implied fraction error minus the model's (units of 2^-53), its sd,
and the distribution of cdf misses in ulps."""
import sys, numpy as np, mpmath, collections, math
from runs import load, groups, lam_of
import lentz as L
M = mpmath.mpf; U = M(2) ** -53
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else 'v/s/s*u*m*x*x'
tf = L.explicit_terms(alf, 'n + w/s + e2*X')
for key, rs in groups(load(case)).items():
    d, ul, its = [], collections.Counter(), []
    for r in rs:
        f, it = L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf); its.append(it)
        ee = (M(r['cdf']) / (M(r['pdf']) * M(r['x'] * r['y'])) / r['rx'] - 1) / U
        d.append(float(ee - (M(f) / r['rx'] - 1) / U))
        m = (r['pdf'] * (r['x'] * r['y'])) * f
        ul[round((m - r['cdf']) / math.ulp(r['cdf']))] += 1
    d = np.array(d)
    print(key, 'terms', min(its), max(its), 'resid mean %+.2f sd %.2f' % (d.mean(), d.std()), 'ulp misses', dict(sorted(ul.items())))
