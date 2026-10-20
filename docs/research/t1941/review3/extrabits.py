# Variant family: the hand-off product P4 carries k extra bits beyond double (k=0 stored, k=11 = x87 64-bit unstored),
# v4 = P4_k + p4 (+ f u), s4 = RNE at u. Same for D's P3 (d3). Per run: best exact rate over f (1/8 grid) for each k.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def rne(v,g):
    q=v/g; f=np.floor(q); r=q-f; up=(r>0.5)|((r==0.5)&(np.mod(f,2)==1)); return (f+up)*g
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
KS=[0,1,2,3,11]; FS=np.arange(-4,5)/8.0
print("run | N: best rate per k (k=0,1,2,3,11) with best f | D: same | stored")
for case in RUNS:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    P4=s*X; uP=ulp(sd(P4)); s4=sd(sd(P4)+LD(P0[4])); u=ulp(s4)
    d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    P3=d*X; uP3=ulp(sd(P3)); d3=sd(sd(P3)+LD(Q0[3])); uD=ulp(d3)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    base=np.mean(yof(s4,d3)==Y); outN=[]; outD=[]
    for k in KS:
        Pk=P4 if k==11 else rne(P4,uP/2**k); vals=[(np.mean(yof(rne(Pk+LD(P0[4])+f*u,u),d3)==Y),f) for f in FS]; outN.append(max(vals))
        Pk3=P3 if k==11 else rne(P3,uP3/2**k); vals=[(np.mean(yof(s4,rne(Pk3+LD(Q0[3])+f*uD,uD))==Y),f) for f in FS]; outD.append(max(vals))
    print(f"{case:6s} x={float(X[n//2]):.3f} | N: "+" ".join(f"k{k}:{r:.3f}({f:+.3f})" for k,(r,f) in zip(KS,outN))+" | D: "+" ".join(f"k{k}:{r:.3f}({f:+.3f})" for k,(r,f) in zip(KS,outD))+f" | {base:.3f}", flush=True)
