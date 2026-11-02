"""Score explicit alpha x beta forms in the Lentz loop (b0 = c*(a/(a+1))) on the tune runs."""
import sys, itertools
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
g = groups(load(case))
res = []
for alf, bef in itertools.product(L.ALPHA_FORMS, L.BETA_FORMS):
    tf = L.explicit_terms(alf, bef); per = []
    for key, rs in g.items():
        per.append(sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf)[0] == r['cdf'] for r in rs))
    res.append((sum(per), alf, bef, per))
for t in sorted(res, reverse=True)[:15]: print(t)
