# Per-run pre-rounding offset c3 at the s3 sum (stored structure, cand9): scan c3 in [-0.5,0.5] ulp(s3) step 1/16, bit-match per run;
# same for d2 (D). Runs: 13 far + near-2 ladders + e404/e405.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
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
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a","e501","e503","r204","e404","e405","e406","e407"]
offs=np.arange(-8,9)/16
print("run     x        best c3 (ulp s3)  score@0 -> best   profile(every 2nd)            | best c2D (ulp d2) score@0 -> best")
for case in RUNS:
    X0,Y0=load(case); m=(X0>2)&(X0<=3); X0=X0[m]; Y0=Y0[m]
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); b=min(b,a+8192)
    X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); D0=chain_off(Q,X,0,0); N0=chain_off(P,X,0,0)
    prof=[float(np.mean(sd(t*sd(chain_off(P,X,3,o)/D0)).astype(np.float64)==Y)) for o in offs]
    profD=[float(np.mean(sd(t*sd(N0/chain_off(Q,X,2,o))).astype(np.float64)==Y)) for o in offs]
    kb=int(np.argmax(prof)); kd=int(np.argmax(profD))
    print(f"{case:6s} {float(X0[a]):.5f}  {offs[kb]:+.3f}            {prof[8]:.4f} -> {prof[kb]:.4f}  "+" ".join(f"{v:.3f}" for v in prof[::2])+f"  | {offs[kd]:+.3f}  {profD[8]:.4f} -> {profD[kd]:.4f}",flush=True)
