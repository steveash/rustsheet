# Where does Excel switch from stored-s3 to exact-s3-like behaviour? Per run near 2: stored vs M* scores and follow-M* fraction vs t = x-2.
import sys, numpy as np, glob, os
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
def hm(c,x,unst):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i])
        if i not in unst: s=sd(s)
    return s
rows=[]
cases=sorted(os.path.basename(f)[len('kmyst-gam-'):-len('.oracle.json')] for f in glob.glob('eval/suites/probes-numeric/cases/kmyst-gam-*.oracle.json'))
for c in cases:
    try: X0,Y0=load(c)
    except Exception: continue
    m=(X0>2)&(X0<2.01)
    if m.sum()<256: continue
    X=X0[m]; Y=Y0[m]; br=[0]+[i+1 for i in range(len(X)-1) if np.nextafter(X[i],np.inf)!=X[i+1]]+[len(X)]
    for a,b in zip(br,br[1:]):
        if b-a<256: continue
        Xl=X[a:b].astype(LD); T=sd(Xl-LD(2)); Yb=Y[a:b]
        yS=sd(T*sd(hm(P,Xl,set())/hm(Q,Xl,set()))).astype(np.float64); yM=sd(T*sd(hm(P,Xl,{3})/hm(Q,Xl,{2}))).astype(np.float64)
        diff=yS!=yM; fm=np.sum(diff&(Yb==yM)); fs=np.sum(diff&(Yb==yS))
        rows.append((float(X[a])-2,b-a,np.mean(yS==Yb),np.mean(yM==Yb),fm/max(fm+fs,1),fm+fs,c))
rows.sort()
print(f"{'t=x-2':>10s} {'n':>6s} {'stored':>7s} {'M*':>7s} {'P(follow M*|diff)':>18s} {'ndiff':>6s} case   log2(t)")
for t,n,a,b,f,nd,c in rows:
    if nd>=20: print(f"{t:10.3e} {n:6d} {a:7.4f} {b:7.4f} {f:18.3f} {nd:6d} {c:7s} {np.log2(t):6.2f}")
