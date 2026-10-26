"""Side check (series region, w3's domain): TOMS bpser with Excel-style prefactors on old direct cells, a,b>1, b*x<=0.7."""
import collections
from common4 import *
import tomsx
from tomsx import mpf, w1
rows = load(lambda d: d['src'] == 'old' and d['branch'] == 'direct' and not math.isnan(d['cdf']) and d['a'] > 1 and d['b'] > 1)
rows = [r for r in rows if tomsx.toms_path(r['x'], r['a'], r['b']) == 'bpser']
tomsx.prefetch('LN', [r['x'] for r in rows])
def bp(xx, a, b, pref):
    lnB = tomsx.lnB_any(a, b)
    LNx = mpf(tomsx.XL('LN', xx))
    if pref == 'stored': z = mpf(w1.D(mpf(w1.D(mpf(a) * LNx)) - mpf(lnB)))
    else: z = w1.R64(mpf(w1.D(mpf(a) * LNx)) - mpf(lnB))
    ans = w1.exp_v6(z) / a
    tol = 1e-15 / a; s = 0.0; c = 1.0; n = 0
    while True:
        n += 1; c = c * ((0.5 - b / n + 0.5) * xx); w = c / (a + n); s += w
        if not (n < 100000 and abs(w) > tol): break
    return ans * (a * s + 1.0)
es = [r['cdf'] for r in rows]
for pref in ('stored', 'ext'):
    ms = [bp(r['x'], r['a'], r['b'], pref) for r in rows]
    print(pref, 'n %d exact %d w4 %d w15 %d' % score(ms, es))
print('CR n %d exact %d w4 %d w15 %d' % score([r['cdfcr'] for r in rows], es))
