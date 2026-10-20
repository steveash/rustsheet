# On R5: a free per-run bias b_P at the hand-off PRODUCT's rounding (ulp(P4), 1/16 steps), N then D; per-run optimum and lift.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
u=2.0**-54; CN=-3128*u; CD=552*u
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
def chain(c,x,K,cc,bP):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        P=s*x
        if i==K: P=P+LD(cc)/x; P=P+bP*ulp(sd(P))
        P=sd(P); v=P+LD(c[i])-(LD(cc) if i==K+1 else 0); s=sd(v)
    return s
B=np.arange(-12,13)/16.0; tot=[]
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2))
    D0=chain(Q0,X,3,CD,0.0)
    rN={bp:np.mean(sd(t*sd(chain(P0,X,4,CN,bp)/D0))==Y) for bp in B}; bN=max(rN,key=rN.get); N1=chain(P0,X,4,CN,bN)
    rD={bp:np.mean(sd(t*sd(N1/chain(Q0,X,3,CD,bp)))==Y) for bp in B}; bD=max(rD,key=rD.get)
    print(f"{case:6s} x={float(X[len(X)//2]):.4f}: R5 {rN[0.0]:.3f} -> N bias {bN:+.4f} ulp(P4) {rN[bN]:.3f} -> +D bias {bD:+.4f} {rD[bD]:.3f}",flush=True); tot.append(rD[bD])
print(f"mean with per-run product biases: {np.mean(tot):.3f}")
