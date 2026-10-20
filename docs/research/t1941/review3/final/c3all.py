# c3 (per-run offset at s3, ulp s3) for EVERY consecutive-double run (>=256 cells) in (2,3] across all cases; save table; study c3(x).
import sys, numpy as np, glob, os, pickle
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
offs=np.arange(-8,9)/16
cases=sorted(os.path.basename(f)[len('kmyst-gam-'):-len('.oracle.json')] for f in glob.glob('eval/suites/probes-numeric/cases/kmyst-gam-*.oracle.json'))
rows=[]; seen=set()
for c in cases:
    try: X0,Y0=load(c)
    except Exception: continue
    m=(X0>2)&(X0<=3)
    if m.sum()<256: continue
    X=X0[m]; Y=Y0[m]; br=[0]+[i+1 for i in range(len(X)-1) if np.nextafter(X[i],np.inf)!=X[i+1]]+[len(X)]
    for a,b in zip(br,br[1:]):
        if b-a<256: continue
        if X[a] in seen: continue
        seen.add(X[a]); b=min(b,a+4096)
        Xl=X[a:b].astype(LD); t=sd(Xl-LD(2)); Yb=Y[a:b]; D0=chain_off(Q,Xl,0,0)
        prof=np.array([np.mean(sd(t*sd(chain_off(P,Xl,3,o)/D0)).astype(np.float64)==Yb) for o in offs]); k=int(np.argmax(prof))
        rows.append((float(X[a]),b-a,offs[k],prof[8],prof[k],c))
rows.sort(); pickle.dump(rows,open('SCRATCH/c3all.pkl','wb'))
R=np.array([(r[0],r[1],r[2],r[3],r[4]) for r in rows]); x=R[:,0]; n=R[:,1]; c3=R[:,2]; s0=R[:,3]; s1=R[:,4]
print(f"{len(rows)} runs; cells {int(n.sum())}; mean score @0 {np.average(s0,weights=n):.4f} -> per-run c3 {np.average(s1,weights=n):.4f}")
inf=(s1-s0)>=0.01
print(f"informative runs (gain>=0.01): {inf.sum()}; c3 distribution: "+" ".join(f"{o:+.3f}:{np.sum(c3[inf]==o)}" for o in offs))
o_=np.argsort(x); xs=x[o_]; cs=c3[o_]; gs=inf[o_]
dx=np.diff(xs); dc=(np.diff(cs)+0.5)%1-0.5; both=gs[:-1]&gs[1:]
for lo,hi in ((0,1e-6),(1e-6,1e-5),(1e-5,1e-4),(1e-4,3e-4),(3e-4,1e-3),(1e-3,3e-3),(3e-3,1e-2),(1e-2,1)):
    k=both&(dx>=lo)&(dx<hi)
    if k.sum(): print(f"   pairs with dx in [{lo:.0e},{hi:.0e}): n={k.sum():4d}  mean|dc| {np.mean(np.abs(dc[k])):.3f} (random=0.25)  coherence |mean e^(2πi dc)| {abs(np.mean(np.exp(2j*np.pi*dc[k]))):.3f}  median dc/dx {np.median(dc[k]/dx[k]):+.0f} cycles/unit")
for r in [r for r in rows if r[4]-r[3]>=0.01][:60]: print(f"   x={r[0]:.6f} n={r[1]:5d} c3={r[2]:+.3f} {r[3]:.3f}->{r[4]:.3f} {r[5]}")
