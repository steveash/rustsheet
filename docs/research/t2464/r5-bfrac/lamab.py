"""a > b runs: which lambda (and y) does Excel's bfrac take? Exact r under candidate forms against Excel's implied r."""
import numpy as np, mpmath
from runs import load, groups
from exact_r import r_exact
M = mpmath.mpf; U = M(2) ** -53
F = {
    'fl((a+b)y) - b   [TOMS a>b]': lambda a, b, x, y: (a + b) * y - b,
    'fl(a - fl((a+b)x))': lambda a, b, x, y: a - (a + b) * x,
    'exact (a+b)(1-x) - b': None,
    '(a+b)y - b, y exact, rounded once': lambda a, b, x, y: float((M(a) + M(b)) * (1 - M(x)) - M(b)),
}
for case in ('t3571-r5-bfrac-tune', 't3571-r5-bfrac-hold'):
    for key, rs in groups(load(case)).items():
        if key[0] <= key[1]: continue
        rs = rs[::3]
        out = []
        for nm, f in F.items():
            e = []
            for r in rs:
                a, b, x, y = r['a'], r['b'], r['x'], r['y']
                if f is None:
                    lam = (M(a) + M(b)) * (1 - M(x)) - M(b)
                    rv = r_exact(a, b, x, y, lam)
                else:
                    rv = r_exact(a, b, x, y, f(a, b, x, y))
                e.append(float((M(r['cdf']) / (M(r['pdf']) * M(x * y)) / rv - 1) / U))
            e = np.array(e); out.append(f'{nm}: {e.mean():+.2f}/{e.std():.2f}')
        print(case, key, ' | '.join(out))
