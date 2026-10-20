# Offset meter on top of keep-mask head models: products of the listed head stages stay in the x87 register (64-bit),
# sums are stored; a smooth offset is added to the (unquantised) product of the hand-off stage before the sum.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def pre(c,k,x,keepm):  # stored-sum Horner through stage k-1 with products in keepm unstored; returns the 80-bit product of stage k
    s=LD(c[0])+0*x
    for i in range(1,k):
        P=s*x
        if i not in keepm: P=sd(P)
        s=sd(P+LD(c[i]))
    return s*x
def finish(c,k,P80,x,quant):
    s=sd((sd(P80) if quant else P80)+LD(c[k]))
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
GRID=np.arange(-112,113)/32.0
def meter(X,Y,kN,kD,keepN,keepD,quantN,quantD,passes=3):
    t=sd(X-LD(2)); PN=pre(P0,kN,X,keepN); PD=pre(Q0,kD,X,keepD)
    uN=ulp(sd(sd(PN)+LD(P0[kN]))); uD=ulp(sd(sd(PD)+LD(Q0[kD])))
    Nc={}; Dc={}
    def Nv(d):
        if d not in Nc: Nc[d]=finish(P0,kN,PN+d*uN,X,quantN)
        return Nc[d]
    def Dv(d):
        if d not in Dc: Dc[d]=finish(Q0,kD,PD+d*uD,X,quantD)
        return Dc[d]
    def rate(dn,dd): return np.mean(sd(t*sd(Nv(dn)/Dv(dd))).astype(np.float64)==Y)
    dn=dd=0.0; base=rate(0.0,0.0)
    for _ in range(passes):
        rn=np.array([rate(d,dd) for d in GRID]); dn=float(GRID[int(np.argmax(rn))])
        rd=np.array([rate(dn,d) for d in GRID]); dd=float(GRID[int(np.argmax(rd))])
    return base,rate(dn,dd),dn,dd
models={
 'stored (base)':            (set(),set(),True,True),
 'Nmul4 kept':               (set(),set(),False,True),
 'Dmul3 kept':               (set(),set(),True,False),
 'Nmul4+Dmul3 kept':         (set(),set(),False,False),
 'Nmul1-4+Dmul1-3 kept':     ({1,2,3},{1,2},False,False),
}
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    for a,b in zip(br,br[1:]):
        if b-a<4096: continue
        X=X0[a:b].astype(LD); Y=Y0[a:b]; x0=float(X[len(X)//2]); out=[]
        for nm,(kn,kd,qn,qd) in models.items():
            base,best,dn,dd=meter(X,Y,4,3,kn,kd,qn,qd); out.append(f"{nm}: {base:.3f}->{best:.3f} ({dn:+.2f},{dd:+.2f})")
        print(f"{case} x0={x0:.4f} | "+" | ".join(out), flush=True)
