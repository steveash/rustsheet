"""b0 forms with the current terms (alpha v/s/s*u*n*(b-n)*x*x, beta with w = n*x*(b-n)), tune runs."""
import sys
from runs import load, groups, lam_of
import lentz as L
g = groups(load(sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'))
for b0f, fn in L.B0_FORMS.items():
    if fn is None: continue
    tf = L.explicit_terms('v/s/s*u*n*(b-n)*x*x', 'R: n + w/s + e2*X', b0f)
    per = [sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf)[0] == r['cdf'] for r in rs) for rs in g.values()]
    print(f'{b0f:14s} {sum(per)} {per}')
