"""a=1 zone a, x>=0.15: for cells needing ly = cr +-1 ulp, how far is exact log1p from the double grid,
and is the needed value explained by the product (b-1)*ly being kept unrounded instead?"""
import sys, collections, math, mpmath
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh and 0.15<=r['x']<=0.375]
ing=ingredients(rows)
LNb=mpf(xl('LN',[rows[0]['b']])[0])
out=collections.Counter()
for r,g in zip(rows,ing):
    ly0=float(D(g['l1p'])); u=math.ulp(ly0)
    frac=float((g['l1p']-mpf(ly0))/u)   # exact - cr in ulps, in [-0.5,0.5]
    ok=[]
    for k in (-1,0,1):
        ly=ly0+k*u
        if exp_v6(R64(mpf(D(g['bm1']*mpf(ly)))+LNb))==r['excel']: ok.append(k)
    tyx=mpf(D(g['bm1']*g['l1p']))   # product of exact log1p, stored
    alt=exp_v6(R64(tyx+LNb))==r['excel']
    out[(tuple(ok), 'near-mid' if abs(frac)>0.4 else 'far', 'exactly1p-ok' if alt else '-')]+=1
for k,v in sorted(out.items(), key=lambda t:-t[1]): print(v,k)
