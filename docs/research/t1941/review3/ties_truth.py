# Class table (as ties.py) for synthetic stored-Horner truths displaced from grid_a along function-null directions,
# scored against grid_a. Truths: early-only (softest directions of the p0..p4/q0..q3 block), early-only without p0,
# the 3rd-softest early direction, and joint all-coefficient 4-softest mixes; scales 30 and 100 ulps (max |move|).
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def ulpc(c): return np.nextafter(abs(c),np.inf)-abs(c)
def upto(c,k,x):
    s=LD(c[0])+0*x
    for i in range(1,k): s=sd(sd(s*x)+LD(c[i]))
    return s*x
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def simy(p,q,X): t=sd(X-LD(2)); return sd(t*sd(tail(p,0,LD(p[0])+0*X,X)/tail(q,0,LD(q[0])+0*X,X))).astype(np.float64)
# null moves
xs=np.linspace(2.01,3.0,600)
def polyval(c,x):
    s=np.zeros_like(x)
    for ci in c: s=s*x+ci
    return s
N=polyval(P0,xs); D=polyval(Q0,xs)
J=np.array([ulpc(c)*xs**(9-i)/N for i,c in enumerate(P0)]+[-ulpc(c)*xs**(8-i)/D for i,c in enumerate(Q0)]).T
names=[f"p{i}" for i in range(10)]+[f"q{i}" for i in range(9)]
rng=np.random.default_rng(1)
def move(colidx,dirs,scale,seed=None):
    U,S,Vt=np.linalg.svd(J[:,colidx],full_matrices=False)
    V=Vt[dirs]; mix=np.random.default_rng(seed).normal(size=len(dirs)) if seed is not None else np.ones(len(dirs))
    v=mix@V; v=v/np.max(np.abs(v))*scale; m=np.zeros(19); m[colidx]=np.round(v); return m
early=[0,1,2,3,4,10,11,12,13]; early_nop0=[1,2,3,4,10,11,12,13]; allc=list(range(19))
truths={
 'early s30 (soft 2)':    move(early,[-1,-2],30,1),
 'early s100 (soft 2)':   move(early,[-1,-2],100,1),
 'early no-p0 s30':       move(early_nop0,[-1,-2],30,2),
 'early no-p0 s100':      move(early_nop0,[-1,-2],100,2),
 'early 3rd-softest s30': move(early,[-3],30),
 'joint4 s30 seed1':      move(allc,[-1,-2,-3,-4],30,1),
 'joint4 s30 seed3':      move(allc,[-1,-2,-3,-4],30,3),
 'joint4 s100 seed1':     move(allc,[-1,-2,-3,-4],100,1),
}
def apply(m): return [c+int(k)*ulpc(c) for c,k in zip(P0,m[:10])],[c+int(k)*ulpc(c) for c,k in zip(Q0,m[10:])]
def classes(X,Y):
    t=sd(X-LD(2)); res={}
    for nm,c,k in (("N",P0,4),("D",Q0,3)):
        P80=upto(c,k,X); Pq=sd(P80); v=Pq+LD(c[k]); s=sd(v); u=ulp(s); q=v/u; fl=np.floor(q); pos=np.array(q-fl,dtype=np.float64)
        sdown=sd(fl*u); sup=sd(fl*u+u)
        if nm=="N":
            Dv=tail(Q0,0,LD(Q0[0])+0*X,X); yd=sd(t*sd(tail(c,k,sdown,X)/Dv)).astype(np.float64); yu=sd(t*sd(tail(c,k,sup,X)/Dv)).astype(np.float64)
        else:
            Nv=tail(P0,0,LD(P0[0])+0*X,X); yd=sd(t*sd(Nv/tail(c,k,sdown,X))).astype(np.float64); yu=sd(t*sd(Nv/tail(c,k,sup,X))).astype(np.float64)
        md=(Y==yd); mu=(Y==yu); dec=md^mu; up=mu&dec; even_up=(np.array(fl%2,dtype=np.float64))!=0
        row=[]
        for cls,sel in (("tie",np.isclose(pos,0.5)),("1/4",np.isclose(pos,0.25)),("3/4",np.isclose(pos,0.75)),("0",np.isclose(pos,0.0))):
            d=sel&dec
            if d.sum()<20: row.append(f"{cls}:  --  "); continue
            rn_up=even_up[d] if cls=="tie" else (pos[d]>0.5)
            row.append(f"{cls}:{np.mean(up[d]==rn_up):.2f}")
        res[nm]=" ".join(row)
    return res
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Yr=Y0[a:b]
    base=simy(list(P0),list(Q0),X)
    r=classes(X,Yr); print(f"== {case} x0={float(X[len(X)//2]):.4f}  REAL exact={np.mean(base==Yr):.3f} | N {r['N']} | D {r['D']}  (agreement of Excel/truth with RN-even per class, decided points)")
    for nm,m in truths.items():
        p,q=apply(m); Y=simy(p,q,X); r=classes(X,Y)
        print(f"   {nm:22s} exact={np.mean(base==Y):.3f} | N {r['N']} | D {r['D']}", flush=True)
print("moves used:")
for nm,m in truths.items(): print(f"   {nm:22s} "+" ".join(f"{n}{int(k):+d}" for n,k in zip(names,m) if k!=0))
