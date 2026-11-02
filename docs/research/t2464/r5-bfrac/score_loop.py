"""Lentz loop options x stop rule on the explicit terms, tune runs."""
import sys, itertools
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else 'v/s/s*u*m*x*x'
g = groups(load(case)); tf = L.explicit_terms(alf, 'R: n + w/s + e2*X')
res = []
for eps, tiny, du, cu, de_, stop in itertools.product((2.220446049250313e-15,), (1e-30, 1e-300), (0, 1), (0, 1), (0, 1, 2), (0, 1, 2)):
    per = []
    for key, rs in g.items():
        per.append(sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf, eps, tiny, du, cu, de_, stop)[0] == r['cdf'] for r in rs))
    res.append((sum(per), eps, tiny, du, cu, de_, stop, per))
for t in sorted(res, reverse=True)[:12]: print(t)
