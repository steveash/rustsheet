"""a<1, b>=8 groups of the old data: shift interval (units) of Excel's E vs M1's E, intersected over the x of a group.
Shows whether M1's lnB (TOMS algdiv) is off by a b-dependent amount."""
import collections, math, mpmath
from lib import load_pdf, mpf, D, R64, exp_v6
from forms import ingredients
from model_final import lnB_model
from ivlib import shift_interval
rows=[r for r in load_pdf() if r['a']<1 and r['b']>=8]
ing=ingredients(rows)
grp=collections.defaultdict(list)
for r,g in zip(rows,ing):
    L=lnB_model(r['a'],r['b'])
    x=r['x']; ly = mpf(D(g['l1p'])) if x <= 0.375 else g['ly_yd']
    tx=mpf(D(g['am1']*g['LNd'])); ty=mpf(D(g['bm1']*ly))
    E=R64(tx+R64(ty-L))
    grp[(r['case'][-12:],r['a'],r['b'])].append(shift_interval(E,r['excel']))
for k in sorted(grp, key=lambda k:(k[1],k[2]))[:80]:
    ivs=[v for v in grp[k] if v]
    lo=max(v[0] for v in ivs); hi=min(v[1] for v in ivs)
    print(k, len(ivs), '[%.2f, %.2f]'%(lo,hi), 'OK0' if lo<=0<=hi else '')
