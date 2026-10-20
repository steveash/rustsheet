# Decouple c_a (the /x term at the hand-off product) from c_b (the -c at the next constant); scan c_a; convert per-run b_P to equivalent c_a.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
u=2.0**-54; CN=-3128*u; CD=552*u
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
data={}
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); data[case]=(X0[a:b].astype(LD),Y0[a:b])
def chain(c,x,K,ca,cb):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        P=s*x
        if i==K: P=P+LD(ca)/x
        P=sd(P); v=P+LD(c[i])-(LD(cb) if i==K+1 else 0); s=sd(v)
    return s
Dcache={c:chain(Q0,data[c][0],3,CD,CD) for c in RUNS}
bias={'r701':-0.25,'r112':0.5,'p401':0.75,'r702':0.75,'r610':0.75,'r901':0.75,'r703':0.375,'r113':-0.0625,'v401a':-0.4375,'r704':-0.125,'r611':-0.5625,'r705':-0.0625,'v407a':0.4375}
for case in RUNS:
    X,Y=data[case]; x=float(X[len(X)//2]); s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    uP=float(ulp(sd(s*X+LD(CN)/X))[0]); print(f"  {case:6s} x={x:.4f} b_P={bias[case]:+.4f} -> equivalent c_a = {CN/u + bias[case]*uP/u*x:+.2f} u")
for ca in np.arange(-3131,-3124.9,0.25):
    r={c:np.mean(sd(sd(X-LD(2))*sd(chain(P0,X,4,ca*u,CN)/Dcache[c]))==Y) for c,(X,Y) in data.items()}
    print(f"  c_a={ca:+.2f}: mean {np.mean(list(r.values())):.3f} | "+" ".join(f"{c}:{v:.3f}" for c,v in r.items()))
