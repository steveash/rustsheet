"""a=1 (C2): if V = D(b*P) for a stored double P, recover the admissible P per cell and compare with EXP forms."""
import sys, math, mpmath, collections
from lib import load_probe, D, R64, exp_v6, mpf, xl, ln64
from forms import ingredients
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
b=rows[0]['b']; bm1=b-1
LNb=mpf(xl('LN',[b])[0])
cnt=collections.Counter()
for r,g in zip(rows,ing):
    V=r['excel']; x=r['x']
    p0=V/b
    Ps=[]
    p=p0
    for k in range(-4,5):
        q=p0
        if k<0:
            for _ in range(-k): q=math.nextafter(q,0)
        else:
            for _ in range(k): q=math.nextafter(q,9)
        if D(mpf(q)*b)==V: Ps.append(q)
    ly = mpf(D(g['l1p'])) if x<=0.375 else g['ly_yd']
    ty = mpf(D(bm1*ly))
    cands={'EXP(D(bm1*ly))':exp_v6(ty),'EXP(e(bm1*ly))':exp_v6(R64(bm1*ly)),'cr pow':float(mpf(1-g['x'])**bm1),
           'EXP(D(b*ly))/y?':None}
    zone='<.375' if x<=0.375 else ('<.5' if x<0.5 else '>=.5')
    cnt[('n',zone)]+=1
    cnt[('hasP',zone)]+=bool(Ps)
    for k,v in cands.items():
        if v is not None: cnt[(k,zone)]+= v in Ps
for k in sorted(cnt): print(k,cnt[k])
