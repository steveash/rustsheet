# Overfit control for the (f, parity) hand-off rule fit: plant a random 24% of s4 +-1 ulp flips on the grid_a chain and refit;
# the spurious gain bounds what a per-run (f,parity) fit can "find" by chance. Also residual class tables one and two stages up.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def rn(v,g,odd):   # round to nearest on grid g with ties to even (odd=False) or odd
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
def chain(c,x,offs,hand,K):   # stored Horner to stage K with pre-rounding offsets offs[i] (ulp of that sum) and the hand-off rule (f, odd) at K
    s=LD(c[0])+0*x
    for i in range(1,K+1):
        v=sd(s*x)+LD(c[i])
        if i in offs: v=v+offs[i]*ulp(sd(v))
        if i==K: g=ulp(sd(v)); s=rn(v+hand[0]*g,g,hand[1])
        else: s=sd(v)
    return s
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
C2=np.arange(-2,3)/4.0; C3=np.arange(-8,9)/8.0; FP=[(f,o) for f in np.arange(-4,4)/8.0 for o in (False,True)]
def jointfit(X,Y,t):
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    s4=chain(P0,X,{},(0.0,False),4); d3=chain(Q0,X,{},(0.0,False),3); base=np.mean(yof(s4,d3)==Y); res=[]
    for c2 in C2:
        for c3 in C3:
            for fp in FP: s4v=chain(P0,X,{2:c2,3:c3},fp,4); res.append((np.mean(yof(s4v,d3)==Y),(c2,c3)+fp,s4v))
    bN=max(res,key=lambda r:r[0]); res=[]
    for g2 in C2:
        for fp in FP: d3v=chain(Q0,X,{2:g2},fp,3); res.append((np.mean(yof(bN[2],d3v)==Y),(g2,)+fp,d3v))
    bD=max(res,key=lambda r:r[0]); return base,bN,bD,yof
rng=np.random.default_rng(7)
for case in ['r112','r113','r610','v401a']:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    base,bN,bD,yof=jointfit(X,Y,t); print(f"== {case} REAL: stored {base:.3f}; joint N {bN[0]:.3f}; +D {bD[0]:.3f}")
    s4=chain(P0,X,{},(0.0,False),4); d3=chain(Q0,X,{},(0.0,False),3); u=ulp(s4)
    flip=rng.random(n)<0.24; sgn=np.where(rng.random(n)<0.5,1.0,-1.0); Yc=yof(sd(s4+np.where(flip,sgn,0.0)*u),d3)
    basec,bNc,bDc,_=jointfit(X,Yc,t); print(f"   PLANTED random 24% flips: stored {basec:.3f}; joint N {bNc[0]:.3f}; +D {bDc[0]:.3f} (spurious gain {bDc[0]-basec:+.3f})")
