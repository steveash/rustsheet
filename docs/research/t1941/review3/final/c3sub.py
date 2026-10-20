# Is the per-run s3 offset constant within a long run? Scan c3 per 512-cell sub-block.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
def chain_off(c,x,stage,off):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        v=sd(s*x)+LD(c[i])
        if i==stage: v=v+off*ulp(sd(v))
        s=sd(v)
    return s
offs=np.arange(-8,9)/16
for case in ["r901","v407a","r701","r703","r112","e501"]:
    X0,Y0=load(case); m=(X0>2)&(X0<=3); X0=X0[m]; Y0=Y0[m]
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); b=min(b,a+8192)
    X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); D0=chain_off(Q,X,0,0)
    ys=[sd(t*sd(chain_off(P,X,3,o)/D0)).astype(np.float64)==Y for o in offs]; out=[]
    for s0 in range(0,b-a,512):
        prof=np.array([np.mean(y[s0:s0+512]) for y in ys]); k=int(np.argmax(prof)); out.append(f"{offs[k]:+.2f}({prof[8]:.2f}->{prof[k]:.2f})")
    print(f"{case:6s} x={float(X0[a]):.5f}: "+" ".join(out),flush=True)
