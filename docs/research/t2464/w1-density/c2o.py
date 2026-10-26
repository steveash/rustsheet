"""a=1: E = S(ty + K) with K in {LN(b) stored, ln64(b), exact ln b, -L1d}, S in {stored, extended}, per zone."""
import sys, math, mpmath, collections, itertools
from lib import load_probe, D, R64, exp_v6, mpf, xl, ln64
from forms import ingredients
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
b=mpf(rows[0]['b']); bm1=b-1
Ks={'LN(b)':mpf(xl('LN',[float(b)])[0]),'ln64(b)':ln64(b),'lnb':mpmath.log(b)}
z=collections.Counter(); n=collections.Counter()
for r,g in zip(rows,ing):
    x=r['x']; zone='a<.375' if x<=0.375 else ('b<.5' if x<0.5 else ('c<.625' if x<0.625 else 'd>=.625'))
    n[zone]+=1
    ly = mpf(D(g['l1p'])) if x<=0.375 else g['ly_yd']
    for kt,ty in {'tyD':mpf(D(bm1*ly)),'tyE':R64(bm1*ly)}.items():
        for (kk,K),s in itertools.product(Ks.items(),'de'):
            E=mpf(D(ty+K)) if s=='d' else R64(ty+K)
            z[(kt,kk,s,zone)]+= exp_v6(E)==r['excel']
zones=sorted(n); print([(zz,n[zz]) for zz in zones])
for k in sorted({k[:3] for k in z}): print(k,[z[k+(zz,)] for zz in zones])
