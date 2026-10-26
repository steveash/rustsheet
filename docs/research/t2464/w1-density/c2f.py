"""a=1: per x-bin hits for ly = crlog1p vs LN(fl(1-x)), fine bins, to locate the switch."""
import sys, math, mpmath, collections
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
sh=sys.argv[1]; w=float(sys.argv[2]) if len(sys.argv)>2 else 0.025
rows=[r for r in load_probe('probe1') if r['case']==sh and r['k']<1000]
ing=ingredients(rows)
LNb=mpf(xl('LN',[rows[0]['b']])[0])
z=collections.defaultdict(lambda:[0,0,0,0])
for r,g in zip(rows,ing):
    b=round(r['x']//w*w,4)
    for j,ly in enumerate((mpf(D(g['l1p'])), g['ly_yd'])):
        ty=mpf(D(g['bm1']*ly)); z[b][j]+= exp_v6(R64(ty+LNb))==r['excel']
    z[b][2]+=1
    z[b][3]+= (mpf(D(g['l1p']))==g['ly_yd'])
for k in sorted(z): print(k, 'log1p',z[k][0],'LN(y)',z[k][1],'n',z[k][2],'same ly',z[k][3])
