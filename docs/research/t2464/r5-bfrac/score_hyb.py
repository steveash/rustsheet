"""Score hybrid_terms (b0 form x e2 form) in the Lentz loop on the tune runs; totals and per-run exact."""
import sys, itertools
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
g = groups(load(case))
res = []
for b0f, e2f in itertools.product(L.B0_FORMS, L.E2_FORMS):
    tf = L.hybrid_terms(b0f, e2f); per = []
    for key, rs in g.items():
        per.append(sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), terms=tf)[0] == r['cdf'] for r in rs))
    res.append((sum(per), b0f, e2f, per))
for t in sorted(res, reverse=True)[:12]: print(t)
print('keys', list(g))
