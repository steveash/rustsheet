"""a=1 (C2): ratio V / P for candidate stored P; a constant K shows as a tight band."""
import sys, math, mpmath, collections
import numpy as np
from lib import load_probe, D, R64, exp_v6, mpf, xl, ln64
from forms import ingredients
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
b=rows[0]['b']; bm1=b-1
U=2.0**-53
for name in ['EXP(D(bm1*ly))','EXP(D(b*ly))/y','exactpow']:
    rr=collections.defaultdict(list)
    for r,g in zip(rows,ing):
        x=r['x']
        ly = mpf(D(g['l1p'])) if x<=0.375 else g['ly_yd']
        if name=='EXP(D(bm1*ly))': P=mpf(exp_v6(mpf(D(bm1*ly))))
        elif name=='EXP(D(b*ly))/y': P=mpf(exp_v6(mpf(D(b*ly))))/(1-g['x'])
        else: P=(1-g['x'])**bm1
        q=float((mpf(r['excel'])/P/b-1)/U)
        zone='<.375' if x<=0.375 else ('<.5' if x<0.5 else '>=.5')
        rr[zone].append(q)
    print(name, {z:(round(np.mean(v),3),round(np.std(v),3),round(min(v),2),round(max(v),2)) for z,v in rr.items()})
