# Tie-class per-anchor offset meter (HANDOFF section 12.3, step 1).
# Model: Excel's chain is grid_a's stored Horner with one smooth sub-ulp offset per chain, injected at stage kN (N) and
# kD (D), everything else exactly as grid_a (quantise the product, add the constant, store, late stored Horner, divide,
# times t). Two injection modes:
#   pre : the offset is added to the 80-bit product of stage k before it is quantised (what an upstream coefficient
#         error looks like at the first stage whose product grid is coarse);
#   post: the offset is added to the constant c[k] (a sub-grid residue error of that constant).
# Offsets are in ulps of the stored sum of that stage. Integer parts are visible through the late chain, so the scan
# covers [-3.5, 3.5] in 1/32 steps by coordinate ascent (3 passes); the objective is the exact-match count.
# Usage: meter.py [--control] [--stage kN kD] [--mode pre|post] cases...
#   --control plants p3 +3, p4 +1, q2 -16 ulps (pre-mode truth) or p5 +1, q4 +2 ulps (post-mode truth) into a synthetic
#   stored-Horner truth and reads it back; the expected readings are printed.
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def ulpc(c): return np.nextafter(abs(c),np.inf)-abs(c)
def pre(c,k,x):            # stored Horner through stage k-1, then the 80-bit product of stage k (before quantisation)
    s=LD(c[0])+0*x
    for i in range(1,k): s=sd(sd(s*x)+LD(c[i]))
    return s*x
def finish(c,k,P80,x,dpost=0.0,u=None):   # quantise the product, add c[k] (+ post offset), store, late stored Horner
    s=sd(sd(P80)+LD(c[k])+(dpost*u if dpost else 0))
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def full(p,q,x): return sd(sd(x-LD(2))*sd(finish(p,1,pre(p,1,x),x)/finish(q,1,pre(q,1,x),x))).astype(np.float64)
GRID=np.arange(-112,113)/32.0
def meter(X,Y,kN,kD,mode,p=P0,q=Q0,passes=3):
    t=sd(X-LD(2)); PN=pre(p,kN,X); PD=pre(q,kD,X)
    uN=ulp(sd(sd(PN)+LD(p[kN]))); uD=ulp(sd(sd(PD)+LD(q[kD])))
    Nc={}; Dc={}
    def Nv(d):
        if d not in Nc: Nc[d]=finish(p,kN,PN+d*uN,X) if mode=='pre' else finish(p,kN,PN,X,d,uN)
        return Nc[d]
    def Dv(d):
        if d not in Dc: Dc[d]=finish(q,kD,PD+d*uD,X) if mode=='pre' else finish(q,kD,PD,X,d,uD)
        return Dc[d]
    def rate(dn,dd): return np.mean(sd(t*sd(Nv(dn)/Dv(dd))).astype(np.float64)==Y)
    dn=0.0; dd=0.0; base=rate(0.0,0.0)
    for _ in range(passes):
        rn=np.array([rate(d,dd) for d in GRID]); dn=float(GRID[int(np.argmax(rn))])
        rdd=np.array([rate(dn,d) for d in GRID]); dd=float(GRID[int(np.argmax(rdd))])
    rn=np.array([rate(d,dd) for d in GRID]); best=max(rate(dn,dd),rn.max()); wn=GRID[rn>=best-0.005]; wd=GRID[rdd>=best-0.005]
    if len(wn)==0: wn=np.array([dn]); 
    if len(wd)==0: wd=np.array([dd])
    return base,best,dn,dd,(wn.min(),wn.max()),(wd.min(),wd.max())
def pert(c,i,k): c=list(c); c[i]=c[i]+k*ulpc(c[i]); return c
if __name__!="__main__": args=[]
else: args=sys.argv[1:]
control=False; kN,kD=4,3; mode='pre'
while args and args[0].startswith('--'):
    if args[0]=='--control': control=True; args=args[1:]
    elif args[0]=='--stage': kN,kD=int(args[1]),int(args[2]); args=args[3:]
    elif args[0]=='--mode': mode=args[1]; args=args[2:]
if __name__=="__main__": print(f"stage ({kN},{kD}) mode {mode}{' CONTROL' if control else ''}: run x0 n | base -> best exact | dN [flat] | dD [flat] | dN/x | dD/x")
for case in args:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    for a,b in zip(br,br[1:]):
        if b-a<4096: continue
        X=X0[a:b].astype(LD); Y=Y0[a:b]; x0=float(X[len(X)//2])
        if control:
            if mode=='pre':   # truth: p3 +3, p4 +1, q2 -16 ulps. At (4,3): dN = 3x*ulp(p3)/u4 + 1, dD = -16x*ulp(q2)/u3
                pt=pert(pert(P0,3,3),4,1); qt=pert(Q0,2,-16); Y=full(pt,qt,X)
                u4=float(ulp(sd(sd(pre(P0,4,X))+LD(P0[4])))[0]); u3=float(ulp(sd(sd(pre(Q0,3,X))+LD(Q0[3])))[0])
                exp=f" | expected at (4,3) dN={3*x0*ulpc(P0[3])/u4+1:+.3f} dD={-16*x0*ulpc(Q0[2])/u3:+.3f}"
            else:             # truth: p5 +1, q4 +2 ulps. At (5,4) post: dN = ulp(p5)/ulp(s5), dD = 2 ulp(q4)/ulp(d4)
                pt=pert(P0,5,1); qt=pert(Q0,4,2); Y=full(pt,qt,X)
                u5=float(ulp(sd(sd(pre(P0,5,X))+LD(P0[5])))[0]); u4d=float(ulp(sd(sd(pre(Q0,4,X))+LD(Q0[4])))[0])
                exp=f" | expected at (5,4) dN={ulpc(P0[5])/u5:+.3f} dD={2*ulpc(Q0[4])/u4d:+.3f}"
        else: exp=""
        base,best,dn,dd,wn,wd=meter(X,Y,kN,kD,mode)
        print(f"{case} x0={x0:.4f} n={len(X)} | {base:.3f} -> {best:.3f} | dN={dn:+.3f} [{wn[0]:+.2f},{wn[1]:+.2f}] | dD={dd:+.3f} [{wd[0]:+.2f},{wd[1]:+.2f}] | {dn/x0:+.3f} | {dd/x0:+.3f}"+exp, flush=True)
