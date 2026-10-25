"""a=1: EXP variants on the extended exponent E = R64(D(bm1*ly) + LN b)."""
import sys, math, mpmath, collections
from lib import load_probe, D, R64, R53, exp_v6, mpf, xl, LOG2E64
from forms import ingredients
def exp_var(E, mode):
    if mode=='cr': return R53(mpmath.exp(E))
    t = R64(E*LOG2E64) if mode!='texact' else E/mpmath.ln(2)
    n = mpmath.nint(t); f = t-n
    if mode in ('neg_direct',):
        u = R64(mpf(2)**f - 1); w = R64(1+u)
    elif mode=='floor':
        n = mpmath.floor(t); f=t-n; u=R64(mpf(2)**f-1); w=R64(1+u)
    else:
        u = R64(mpf(2)**abs(f)-1); w=R64(1+u)
        if f<0: w=R64(1/w)
    return R53(w*mpf(2)**n)
sh=sys.argv[1]
rows=[r for r in load_probe('probe1') if r['case']==sh]
ing=ingredients(rows)
b=rows[0]['b']; bm1=b-1
LNb=mpf(xl('LN',[b])[0])
z=collections.Counter(); n=collections.Counter()
for r,g in zip(rows,ing):
    x=r['x']; zone='<.375' if x<=0.375 else ('<.5' if x<0.5 else '>=.5')
    ly = mpf(D(g['l1p'])) if x<=0.375 else g['ly_yd']
    E=R64(mpf(D(bm1*ly))+LNb)
    n[zone]+=1
    for mode in ('v6','cr','texact','neg_direct','floor'):
        z[(mode,zone)]+= exp_var(E,mode)==r['excel']
for k in sorted(z): print(k,z[k],'/',n[k[1]])
