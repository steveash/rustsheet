"""Stop threshold scan for the Lentz loop (explicit terms, plain loop), tune runs: |C*D - 1| <= eps."""
import sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else '(u/s)*(v/s)*(w*x)'
g = groups(load(case)); tf = L.explicit_terms(alf, 'n + w/s + e2*X')
for eps in (1e-15, 1.5e-15, 2e-15, 2.1e-15, 2.220446049250313e-15, 2.3e-15, 2.5e-15, 3e-15):
    per = [sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), tf, eps)[0] == r['cdf'] for r in rs) for rs in g.values()]
    print(f'eps {eps:.6g}: {sum(per)} {per}')
