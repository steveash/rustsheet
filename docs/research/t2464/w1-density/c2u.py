"""a=1 zone a: score log1p implementations for ly in E = R64(D(bm1*ly) + LN b) (and stored E), per x bin."""
import sys, collections, math, mpmath
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
from log1p_impls import IMPLS
for sh in sys.argv[1:]:
    rows=[r for r in load_probe('probe1') if r['case']==sh and r['x']<=0.375]
    ing=ingredients(rows)
    LNb=mpf(xl('LN',[rows[0]['b']])[0])
    c=collections.Counter()
    for r,g in zip(rows,ing):
        key='x<.15' if r['x']<0.15 else 'x>=.15'
        for name,f in IMPLS.items():
            ly=mpf(f(-r['x']))
            ty=mpf(D(g['bm1']*ly))
            c[(name,'e',key)]+= exp_v6(R64(ty+LNb))==r['excel']
            c[(name,'d',key)]+= exp_v6(mpf(D(ty+LNb)))==r['excel']
    n=collections.Counter('x<.15' if r['x']<0.15 else 'x>=.15' for r in rows)
    print(sh, dict(n))
    for name in IMPLS: print('  %-14s'%name, 'E ext', c[(name,'e','x<.15')], c[(name,'e','x>=.15')], '| E stored', c[(name,'d','x<.15')], c[(name,'d','x>=.15')])
