"""Is the density the cdf's prefactor divided by x (and y)? b = 1 sheets: cdf = EXP(a*LN(x)) (T-1744 closed form);
test a*cdf/x orders. General sheets: prefactor EXP(a lnx + b lny - lnB)/(x*y) orders."""
import collections, mpmath
from lib import load_probe, D, R64, exp_v6, mpf
from forms import ingredients
from model_final import lnB_model
for sh in ('probe1C1','probe1C8','probe1C0','probe1C3','probe1C7'):
    rows=[r for r in load_probe('probe1') if r['case']==sh]
    ing=ingredients(rows); L=lnB_model(rows[0]['a'],rows[0]['b'])
    c=collections.Counter()
    for r,g in zip(rows,ing):
        a,b,x=g['a'],g['b'],g['x']; y=mpf(1.0-r['x'])
        ly = mpf(D(g['l1p'])) if r['x'] <= 0.375 else g['ly_yd']
        za=mpf(D(a*g['LNd'])); zb=mpf(D(b*ly))
        if r['b']==1.0:
            cdf=mpf(exp_v6(za))
            c['D(D(a*cdf)/x)']+= D(mpf(D(a*cdf))/x)==r['excel']
            c['D(a*D(cdf/x))']+= D(a*mpf(D(cdf/x)))==r['excel']
        for kz,E in (('ext',R64(za+R64(zb-L))),('stored',mpf(D(za+mpf(D(zb-L)))))):
            P=mpf(exp_v6(E))
            c[f'pref {kz} /x/y']+= D(mpf(D(P/x))/y)==r['excel']
            c[f'pref {kz} /(x*y)']+= D(P/mpf(D(x*y)))==r['excel']
    print(sh, rows[0]['a'], rows[0]['b'], dict(c))
