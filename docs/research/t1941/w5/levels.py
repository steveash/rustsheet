import mpmath as mp, sys
from remez import remez
def f(x): return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
for m in [5,6,7,8,9]:
    p,q,E,X=remez(f,0.7,1.5,m,m,dps=50,grid=1500)
    print(m,mp.nstr(E,5),flush=True)
