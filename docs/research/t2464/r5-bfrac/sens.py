"""Regress Excel's implied fraction error on structural rounding terms: the exact r's sensitivity to each fraction
input times that input's double rounding (y = fl(1-x), yp1 = fl(y+1), c = fl(1+lambda), lambda = fl(a - fl((a+b)x)),
and the pdf's own error). Coefficient 0 = Excel rounds that input as modelled; -1 = Excel uses it unrounded."""
import numpy as np, mpmath
from fractions import Fraction as F
from exact_r import get
mpmath.mp.dps = 40
M = mpmath.mpf; U = M(2) ** -53
def rfrac(a, b, x, y, lam, yp1=None, c=None, nmax=60):
    c = 1 + lam if c is None else c; yp1 = y + 1 if yp1 is None else yp1
    c0 = b / a; c1 = 1 + 1 / a; n = M(0); p = M(1); s = a + 1
    v = M(0); T = []
    for _ in range(nmax):
        n += 1; t = n / a; w = n * (b - n) * x; e = a / s
        al = p * (p + c0) * e * e * (w * x); e = (1 + t) / (c1 + t + t); be = n + w / s + e * (c + n * yp1)
        p = 1 + t; s += 2; T.append((al, be))
    for al, be in reversed(T): v = al / (be + v)
    return 1 / ((c) / c1 + v)
rows = get()
ee, X = [], []
h = M(10) ** -20
for r in rows:
    a, b, x = M(r['a']), M(r['b']), M(r['x'])
    yd = 1 - r['x']; lamd = r['a'] - (r['a'] + r['b']) * r['x']; yp1d = yd + 1.0; cd = 1.0 + lamd
    y, lam = M(yd), M(lamd)
    base = rfrac(a, b, x, y, lam, M(yp1d), M(cd))
    ee.append(float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * yd)) / base - 1) / U))
    row = []
    # each: relative change in r when the input takes its unrounded value
    row.append(float((rfrac(a, b, x, 1 - x, lam, M(yp1d), M(cd)) / base - 1) / U))         # y exact in fraction (w only via yp1 here: none) -> 0
    row.append(float((rfrac(a, b, x, y, lam, y + 1, M(cd)) / base - 1) / U))               # yp1 unrounded
    row.append(float((rfrac(a, b, x, y, lam, M(yp1d), 1 + lam) / base - 1) / U))           # c unrounded
    row.append(float((rfrac(a, b, x, y, a - (a + b) * x, M(yp1d), 1 + a - (a + b) * x) / base - 1) / U))  # lambda exact
    row.append(float((rfrac(a, b, x, 1 - x, lam, 2 - x, M(cd)) / base - 1) / U))           # yp1 from exact y
    X.append(row)
ee = np.array(ee); X = np.array(X)
names = ['y exact', 'yp1 unrounded', 'c unrounded', 'lambda exact', 'yp1=2-x exact']
for i, nm in enumerate(names):
    v = X[:, i]
    if v.std() == 0: print(nm, 'no variation'); continue
    k = np.polyfit(v, ee, 1)[0]
    print(f'{nm:15s} sd {v.std():.2f} corr {np.corrcoef(v, ee)[0,1]:+.3f} slope {k:+.2f}')
print('ee mean %+.2f sd %.2f' % (ee.mean(), ee.std()))
