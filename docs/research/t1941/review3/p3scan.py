# Per-run 1-D scans of single early coefficients (ulps) with everything else fixed at grid_a: where is each run's optimum?
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def chain(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def yv(p,q,x): return sd(sd(x-LD(2))*sd(chain(p,x)/chain(q,x))).astype(np.float64)
def pert(c,i,k): c=list(c); c[i]=c[i]+k*(np.nextafter(c[i],np.inf)-c[i]); return c
K=range(-24,25)
print("run x0 | best k and rate for: p3 | p2 | p4 | q2 | q3   (base rate first)")
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]
    base=np.mean(yv(P0,Q0,X)==Y); out=[]
    for nm,ci,isq in (("p3",3,False),("p2",2,False),("p4",4,False),("q2",2,True),("q3",3,True)):
        rates=[]
        for k in K:
            pp=P0 if isq else pert(P0,ci,k); qq=pert(Q0,ci,k) if isq else Q0
            rates.append(np.mean(yv(pp,qq,X)==Y))
        j=int(np.argmax(rates)); out.append(f"{nm}: k={K[j]:+d} {rates[j]:.3f}")
    print(f"{case} x0={float(X[len(X)//2]):.4f} base={base:.3f} | "+" | ".join(out))
