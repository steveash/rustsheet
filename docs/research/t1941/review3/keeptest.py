# Conditional agreement test: for each keep-mask variant V of the stored x87 chain (grid_a), among points where V != base,
# what fraction does Excel follow V? 0.5 = no information, >>0.5 favours V, <<0.5 favours base.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def chain(c,x,keep):  # keep: set of ('mul',i) / ('add',i) left unstored (80-bit)
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=s*x
        if ('mul',i) not in keep: s=sd(s)
        s=s+LD(c[i])
        if ('add',i) not in keep: s=sd(s)
    return s
def y(p,q,x,kN,kD,kdiv=False):
    n=chain(p,x,kN); d=chain(q,x,kD); t=sd(x-LD(2)); r=n/d
    if not kdiv: r=sd(r)
    return sd(t*r).astype(np.float64)
muls=lambda a,b:{('mul',i) for i in range(a,b+1)}; adds=lambda a,b:{('add',i) for i in range(a,b+1)}
variants={
 'Nmul4':(muls(4,4),set()), 'Nmul3':(muls(3,3),set()), 'Nmul5':(muls(5,5),set()), 'Nmul1-4':(muls(1,4),set()),
 'Dmul3':(set(),muls(3,3)), 'Dmul2':(set(),muls(2,2)), 'Dmul4':(set(),muls(4,4)), 'Dmul1-3':(set(),muls(1,3)),
 'headprods N1-4,D1-3':(muls(1,4),muls(1,3)), 'allprods':(muls(1,9),muls(1,8)),
 'head80 N(1-4),D(1-3) [m8]':(muls(1,4)|adds(1,3),muls(1,3)|adds(1,2)),
 'Nadd4':(adds(4,4),set()), 'Dadd3':(set(),adds(3,3)), 'Nadd3':(adds(3,3),set()), 'Nadd4+Dadd3':(adds(4,4),adds(3,3)),
 'all80':(muls(1,9)|adds(1,9),muls(1,8)|adds(1,8)),
}
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]
    base=y(P0,Q0,X,set(),set()); print(f"{case} n={len(X)} base exact={np.mean(base==Y):.4f}")
    for nm,(kN,kD) in variants.items():
        v=y(P0,Q0,X,kN,kD); diff=v!=base; 
        if diff.sum()==0: print(f"   {nm:28s} identical to base"); continue
        follow=np.mean(Y[diff]==v[diff]); followb=np.mean(Y[diff]==base[diff])
        print(f"   {nm:28s} exact={np.mean(v==Y):.4f}  differs on {diff.mean():.3f}; Excel follows V {follow:.3f} / base {followb:.3f} / neither {1-follow-followb:.3f}")
