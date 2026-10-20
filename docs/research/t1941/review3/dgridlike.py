# dgrid-style per-cluster smooth hand-off offset scan at (4,3): 80-bit head + fractional offset (in ulps of the stored head),
# stored, then grid_a's stored late chain. Scored on REAL Excel and on a synthetic all-stored grid_a truth (base ceiling).
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def head80(c,k,x):
    s=LD(c[0])+0*x
    for i in range(1,k+1): s=s*x+LD(c[i])
    return s
def headst(c,k,x):
    s=LD(c[0])+0*x
    for i in range(1,k+1): s=sd(sd(s*x)+LD(c[i]))
    return s
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def full(p,q,x): return sd(sd(x-LD(2))*sd(tail(p,0,LD(p[0])+0*x,x)/tail(q,0,LD(q[0])+0*x,x))).astype(np.float64)
G=24; R=8  # +-3 ulps in 1/8 steps
offs=np.arange(-G,G+1)/R
def scan(X,Y,CL=512):
    t=sd(X-LD(2)); EN=head80(P0,4,X); ED=head80(Q0,3,X); uN=ulp(sd(EN)); uD=ulp(sd(ED))
    tot=0; best=0; at0=0
    for a in range(0,len(X)-CL+1,CL):
        sl=slice(a,a+CL); x=X[sl]; y=Y[sl]
        nn=np.stack([tail(P0,4,sd(EN[sl]+o*uN[sl]),x) for o in offs]); dd=np.stack([tail(Q0,3,sd(ED[sl]+o*uD[sl]),x) for o in offs])
        cnt=np.zeros((len(offs),len(offs)),int)
        for i in range(len(offs)):
            yy=sd(t[sl]*sd(nn[i][None,:]/dd)).astype(np.float64); cnt[i]=(yy==y[None,:]).sum(1)
        best+=cnt.max(); at0+=cnt[G,G]; tot+=CL
    return at0/tot, best/tot
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]
    r0,rb=scan(X,Y); Ys=full(P0,Q0,X); s0,sb=scan(X,Ys)
    print(f"{case} x0={float(X[len(X)//2]):.4f}  REAL: at(0,0) {r0:.3f} best-per-cluster {rb:.3f}   |  BASE truth (grid_a stored): at(0,0) {s0:.3f} best {sb:.3f}")
