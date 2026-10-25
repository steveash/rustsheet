"""a=1: scan a constant exponent offset c (units 2^-53) before the store, D(Ee + c) and Ee + c, per zone."""
import sys, math, mpmath, collections
import numpy as np
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
U=mpf(2)**-53
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
LNb=mpf(xl('LN',[rows[0]['b']])[0])
Es=[]
for r,g in zip(rows,ing):
    x=r['x']
    ly = mpf(D(g['l1p'])) if x<=0.375 else g['ly_yd']
    Es.append((R64(mpf(D(g['bm1']*ly))+LNb), r['excel'], 'a' if x<=0.375 else ('b' if x<0.5 else ('c' if x< 0.625 else 'd'))))
for c in np.arange(-1.5,1.51,0.125):
    hd=collections.Counter(); he=collections.Counter()
    for E,v,z in Es:
        hd[z]+= exp_v6(mpf(D(E+c*U)))==v
        he[z]+= exp_v6(R64(E+c*U))==v
    print('%+.3f'%c, 'stored', [hd[z] for z in 'abcd'], 'ext', [he[z] for z in 'abcd'])
