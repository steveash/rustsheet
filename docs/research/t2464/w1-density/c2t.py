"""a=1 zone a: which ulp offsets k of ly = D(log1p(-x)) + k ulp make each cell exact (E extended / stored)."""
import sys, collections, math, mpmath
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
sh=sys.argv[1]; mode=sys.argv[2]
rows=[r for r in load_probe('probe1') if r['case']==sh and r['x']<=0.375]
ing=ingredients(rows)
LNb=mpf(xl('LN',[rows[0]['b']])[0])
stat=collections.Counter(); per=collections.defaultdict(collections.Counter)
for r,g in zip(rows,ing):
    ly0=float(D(g['l1p']))
    ok=[]
    for k in range(-6,7):
        ly=ly0
        for _ in range(abs(k)): ly=math.nextafter(ly, 1 if k>0 else -1)
        ty=mpf(D(g['bm1']*mpf(ly)))
        E= R64(ty+LNb) if mode=='e' else mpf(D(ty+LNb))
        if exp_v6(E)==r['excel']: ok.append(k)
    # error of cr log1p in ulps of ly
    e=float((mpf(ly0)-g['l1p'])/math.ulp(ly0))
    key='x<.15' if r['x']<0.15 else 'x>=.15'
    stat[(key, tuple(ok))]+=1
for k,v in sorted(stat.items(), key=lambda t:-t[1])[:25]: print(v,k)
