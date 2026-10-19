# how well do (k,k) rationals approximate c5_f99t's exact function on [0.7,1.5]?
import mpmath as mp
from remez import remez
from sc import candidate
N=[mp.mpf(v) for v in candidate('N3')[::-1]]; D=[mp.mpf(v) for v in candidate('D3')[::-1]]
def g(x): return mp.polyval(N,x)/mp.polyval(D,x)
for m in [5,6,7,8]:
    p,q,E,X=remez(g,0.7,1.5,m,m,dps=50,grid=1500)
    print(m,mp.nstr(E,5),flush=True)
