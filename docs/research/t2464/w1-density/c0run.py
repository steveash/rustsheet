"""C0 run x~0.0123: needed shift (vs exact-ingredient exponent) next to candidate ingredient errors."""
import sys, math, mpmath
from lib import load_probe, D, R64, exp_v6, mpf, xl, ln64
from forms import ingredients
from ivlib import shift_interval, U
sh=sys.argv[1]; k0,k1=int(sys.argv[2]),int(sys.argv[3])
rows=[r for r in load_probe('probe1') if r['case']==sh and k0<=r['k']<k1]
ing=ingredients(rows)
for r,g in zip(rows,ing):
    x=g['x']
    ga,gb,gab=g['ga'],g['gb'],g['gab']
    L=mpf(D(ga+D(gb-gab)))
    Eex=(g['a']-1)*mpmath.log(x)+(g['b']-1)*mpmath.log1p(-x)-L   # exact logs, Excel's L
    iv=shift_interval(Eex, r['excel'])
    eLN=float((g['am1']*g['LNd']-(g['a']-1)*mpmath.log(x))/U)
    etx=float((mpf(D(g['am1']*g['LNd']))-(g['a']-1)*mpmath.log(x))/U)
    print(r['k'], 'need [%.2f,%.2f]'%iv, 'am1*errLN %.2f'%eLN, 'err D(tx) %.2f'%etx)
