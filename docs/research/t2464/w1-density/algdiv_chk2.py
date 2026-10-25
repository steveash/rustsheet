"""Score algdiv variants on old a<1, b>8 groups (and b=8 with the GL form): groups whose E-shift intersection holds 0,
and cell hits."""
import collections, mpmath
from lib import load_pdf, mpf, D, R64, exp_v6
from forms import ingredients
from toms_betaln import GL, algdiv, algdiv_x87
from ivlib import shift_interval
rows=[r for r in load_pdf() if r['a']<1 and r['b']>=8]
ing=ingredients(rows)
def L_of(a,b,v):
    ga=mpf(GL(a))
    if b<=8 and v.endswith('+b<=8GL'): return mpf(D(ga+D(mpf(GL(b))-GL(a+b))))
    if v.startswith('dbl'): return mpf(D(ga+mpf(algdiv(a,b))))
    if v.startswith('x87s'): return mpf(D(ga+mpf(algdiv_x87(a,b,True))))
    if v.startswith('x87e'): return R64(ga+algdiv_x87(a,b,False))
    if v.startswith('x87e_lnD'): return R64(ga+algdiv_x87(a,b,False,'D'))
    if v.startswith('x87e_alD'): return R64(ga+algdiv_x87(a,b,False,'ln64','dbl'))
V=['dbl','dbl+b<=8GL','x87s+b<=8GL','x87e+b<=8GL','x87e_lnD+b<=8GL','x87e_alD+b<=8GL']
for v in V:
    hits=0; gok=collections.Counter(); grp=collections.defaultdict(list)
    for r,g in zip(rows,ing):
        L=L_of(r['a'],r['b'],v)
        x=r['x']; ly = mpf(D(g['l1p'])) if x <= 0.375 else g['ly_yd']
        tx=mpf(D(g['am1']*g['LNd'])); ty=mpf(D(g['bm1']*ly))
        E=R64(tx+R64(ty-L)); m=exp_v6(E); hits+= m==r['excel']
    print(v, 'hits', hits, '/', len(rows))
