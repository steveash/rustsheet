# Per-point admissible (dN, dD) shifts at the (4,3) hand-off with the late chain exact (grid_a, optional needle corr.),
# and the phase-locking of the points that need a shift, against the s4/d3/s5/d4 sawtooth phases.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
corr=int(sys.argv[1]); cases=sys.argv[2:]
p=list(P0); q=list(Q0)
if corr:
    p[5]=p[5]-4*(np.nextafter(p[5],np.inf)-p[5]); q[4]=q[4]-1*(np.nextafter(q[4],np.inf)-q[4])
def head(c,k,x):
    s=LD(c[0])+0*x; vals=[]
    for i in range(1,k+1):
        P80=s*x; Pst=sd(P80); s=sd(Pst+LD(c[i])); vals.append((P80,Pst,s))
    return s, vals
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
for case in cases:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; n=len(X)
    t=sd(X-LD(2))
    s4,vN=head(p,4,X); d3,vD=head(q,3,X); u4=ulp(s4); u3=ulp(d3)
    adm={}
    for da in range(-2,3):
        nn=tail(p,4,sd(s4+da*u4),X)
        for db in range(-2,3):
            dd=tail(q,3,sd(d3+db*u3),X); y=sd(t*sd(nn/dd)).astype(np.float64); adm[(da,db)]=(y==Y)
    ok00=adm[(0,0)]; need=~ok00
    # minimal |shift| needed
    minN=np.full(n,9); minD=np.full(n,9); anyok=np.zeros(n,bool)
    for (da,db),m in adm.items():
        anyok|=m; sel=m&(np.abs(da)+np.abs(db)<minN+minD); minN=np.where(sel,abs(da),minN); minD=np.where(sel,abs(db),minD)
    print(f"{case} n={n} corr={corr} exact(0,0)={ok00.mean():.3f} explained by |a|,|b|<=2: {anyok.mean():.3f}; needing shift: {need.mean():.3f}; of those minimal (|a|,|b|) histogram:")
    from collections import Counter
    cnt=Counter(zip(minN[need&anyok],minD[need&anyok])); tot=(need&anyok).sum()
    print("    "+"  ".join(f"({a_},{b_}):{c/tot:.2f}" for (a_,b_),c in sorted(cnt.items(),key=lambda kv:-kv[1])[:8]))
    # phases
    P4_80,P4st,_=vN[3]; phi4=np.array((P4_80/u4)%1,dtype=np.float64)            # s4 sum sawtooth phase (frac of P4/u)
    P3_80,P3st,_=vD[2]; phiD3=np.array((P3_80/u3)%1,dtype=np.float64)
    P5=sd(s4*X); s5=sd(P5+LD(p[5])); u5=ulp(s5); phi5=np.array(((P5+LD(p[5]))/u5)%1,dtype=np.float64)
    P4D=sd(d3*X); d4=sd(P4D+LD(q[4])); u4D=ulp(d4); phiD4=np.array(((P4D+LD(q[4]))/u4D)%1,dtype=np.float64)
    def lock(name,phi,sel):
        h,_=np.histogram(phi[sel],bins=8,range=(0,1)); hall,_=np.histogram(phi,bins=8,range=(0,1))
        r=h/np.maximum(hall,1)/max(sel.mean(),1e-9)
        print(f"    {name:12s} need-shift rate by phase octile (relative to mean): "+" ".join(f"{v:4.2f}" for v in r))
    lock("phi(s4 sum)",phi4,need); lock("phi(d3 sum)",phiD3,need); lock("phi(s5 sum)",phi5,need); lock("phi(d4 sum)",phiD4,need)
    # N-only vs D-only shifts
    nOnly=need&adm[(1,0)]|need&adm[(-1,0)]; dOnly=need&adm[(0,1)]|need&adm[(0,-1)]
    print(f"    fixable by N+-1 alone: {nOnly.mean():.3f}; by D+-1 alone: {dOnly.mean():.3f}")
    lock("phi(s4)|Nfix",phi4,nOnly); lock("phi(s5)|Nfix",phi5,nOnly); lock("phi(d3)|Dfix",phiD3,dOnly); lock("phi(d4)|Dfix",phiD4,dOnly)
