"""Enumerate beta_n forms: beta = sum of N = n, Wd (w/s variants), E (e2*X variants), X (c + n*yp1 variants), in
every association order; alpha = argv[2] form, Lentz loop, b0 as modelled. Prints the top forms by tune exact."""
import itertools, sys
from runs import load, groups, lam_of
import lentz as L
case = sys.argv[1] if len(sys.argv) > 1 else 't3571-r5-bfrac-tune'
alf = sys.argv[2] if len(sys.argv) > 2 else 'v/s/s*u*m*x*x'
g = list(groups(load(case)).values())
WD = {'w/s': lambda n, m, w, s, x, b: w / s, 'm/s*x': lambda n, m, w, s, x, b: m / s * x, 'x*m/s': lambda n, m, w, s, x, b: x * m / s,
      'n*x*(b-n)/s': lambda n, m, w, s, x, b: n * x * (b - n) / s, 'w*(1/s)': lambda n, m, w, s, x, b: w * (1.0 / s),
      'n/s*(b-n)*x': lambda n, m, w, s, x, b: n / s * (b - n) * x, 'm*(x/s)': lambda n, m, w, s, x, b: m * (x / s)}
XS = {'c+n*yp1': lambda c, n, y, lam: c + n * (y + 1.0), 'c+n+n*y': lambda c, n, y, lam: c + n + n * y,
      'c+(n+n*y)': lambda c, n, y, lam: c + (n + n * y), 'lam+(1+n*yp1)': lambda c, n, y, lam: lam + (1.0 + n * (y + 1.0)),
      '(c+n)+n*y': lambda c, n, y, lam: (c + n) + n * y, 'lam+n*yp1+1': lambda c, n, y, lam: lam + n * (y + 1.0) + 1.0,
      'c+n*(1+y)': lambda c, n, y, lam: c + n * (1.0 + y)}
ES = {'e2*X': lambda a, n, X: (a + n) / (a + 2 * n + 1) * X, '(a+n)*X/(a+2n+1)': lambda a, n, X: (a + n) * X / (a + 2 * n + 1),
      'X/((a+2n+1)/(a+n))': lambda a, n, X: X / ((a + 2 * n + 1) / (a + n)), '(a+n)*(X/(a+2n+1))': lambda a, n, X: (a + n) * (X / (a + 2 * n + 1))}
SUMS = {'N+W+E': lambda N, W, E: N + W + E, 'W+(N+E)': lambda N, W, E: W + (N + E), 'N+(W+E)': lambda N, W, E: N + (W + E),
        'N+E+W': lambda N, W, E: N + E + W, 'W+E+N': lambda N, W, E: W + E + N}
res = []
for (wn, wf), (xn, xf), (en, ef), (sn, sf) in itertools.product(WD.items(), XS.items(), ES.items(), SUMS.items()):
    def mk(a, b, x, y, lam, wf=wf, xf=xf, ef=ef, sf=sf):
        c = lam + 1.0; b0 = L.B0_FORMS['c*(a/(a+1))'](a, c)
        def gen():
            n = 0.0
            while True:
                n += 1.0; u = a + n - 1.0; v = a + b + n - 1.0; s = a + 2.0 * n - 1.0; m = n * (b - n); w = m * x
                al = L.ALPHA_FORMS[alf](u, v, s, m, w, x, a, b, n)
                yield al, sf(n, wf(n, m, w, s, x, b), ef(a, n, xf(c, n, y, lam)))
        return b0, gen()
    per = [sum((r['pdf'] * (r['x'] * r['y'])) * L.lentz2(r['a'], r['b'], r['x'], r['y'], lam_of(r['a'], r['b'], r['x'], r['y']), mk)[0] == r['cdf'] for r in rs) for rs in g]
    res.append((sum(per), wn, xn, en, sn, per))
for t in sorted(res, reverse=True)[:12]: print(t)
