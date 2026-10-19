# Convergence of the x=0 invariants of a minimax parent with Remez tolerance / precision.
import mpmath as mp
from remez2 import remez2
from taylor0 import taylor
def A(x): return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
for dps,tol,grid in [(40,'1e-10',500),(50,'1e-14',500),(60,'1e-18',1000)]:
    p,q,E,X=remez2(A,0.7,1.5,8,8,dps=dps,tol=tol,grid=grid,iters=40)
    print(dps,tol,grid,[mp.nstr(v,16) for v in taylor(p,q)],flush=True)
