"""T-3571: how many hold-out and fresh cells change between the ends of EPS's bracket (2.1e-15, 2.5e-15) and 10*DBL_EPSILON,
and the bit-exact count at each, with model.py's loop otherwise unchanged."""
import model
from runs import load
for case in ('t3571-r5-bfrac-hold', 't3571-r5-bfrac-fresh'):
    rows = load(case); out = {}
    for eps in (2.1e-15, model.EPS, 2.5e-15):
        out[eps] = [(r['pdf'] * (r['x'] * (1 - r['x']))) * model.fraction(r['a'], r['b'], r['x'], 1 - r['x'], r['a'] - (r['a'] + r['b']) * r['x'], eps)[0] for r in rows]
    for eps, v in out.items():
        print(f'{case} eps {eps:.4g}: exact {sum(m == r["cdf"] for m, r in zip(v, rows))}, cells differing from 10*DBL_EPSILON {sum(m != n for m, n in zip(v, out[model.EPS]))} of {len(rows)}')
