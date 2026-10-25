"""a=1: per-zone table of ly / ty variants (E = R64(ty + LN b))."""
import sys, math, mpmath, collections, itertools
from lib import load_probe, D, R64, exp_v6, mpf, xl, ln64
from forms import ingredients
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
b=mpf(rows[0]['b']); bm1=b-1
LNb=mpf(xl('LN',[float(b)])[0])
z=collections.Counter(); n=collections.Counter()
for r,g in zip(rows,ing):
    x=r['x']; zone='a<.375' if x<=0.375 else ('b<.5' if x<0.5 else ('c<.625' if x<0.625 else 'd>=.625'))
    n[zone]+=1
    lys={'D(log1p)':mpf(D(g['l1p'])),'LN(y)':g['ly_yd'],'ln64(y)':ln64(mpf(1.0-r['x'])),'ln64(1-x)':g['ly_e']}
    for (kl,ly) in lys.items():
        tys={'D(bm1*ly)':mpf(D(bm1*ly)),'E(bm1*ly)':R64(bm1*ly),'D(D(b*ly)-ly)':mpf(D(mpf(D(b*ly))-ly)),'E(E(b*ly)-ly)':R64(R64(b*ly)-ly)}
        for kt,ty in tys.items():
            z[(kl,kt,zone)]+= exp_v6(R64(ty+LNb))==r['excel']
zones=sorted(n)
keys=sorted({k[:2] for k in z})
print('zones',[(zz,n[zz]) for zz in zones])
for k in keys: print(f"{k[0]:10s} {k[1]:14s}", [z[k+(zz,)] for zz in zones])
