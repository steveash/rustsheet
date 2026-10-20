# Score a set on ALL unique x in (2,3] across the 1,089 cases, by family; saves allXY.npy for cand8/cand9.
import sys, numpy as np, glob, os, re, collections
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
cases=sorted(os.path.basename(f)[len('kmyst-gam-'):-len('.oracle.json')] for f in glob.glob('eval/suites/probes-numeric/cases/kmyst-gam-*.oracle.json'))
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
UN_N=set(); UN_D=set()   # stored structure ({3},{2} = M*)
def hm(c,x,unst):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i])
        if i not in unst: s=sd(s)
    return s
def ev(P,Q,X,uN,uD): Xl=X.astype(LD); return sd(sd(Xl-LD(2))*sd(hm(P,Xl,uN)/hm(Q,Xl,uD))).astype(np.float64)
seen=set(); fam=collections.defaultdict(lambda:[0,0,0]); tot=[0,0,0]; Xall=[]; Yall=[]
for c in cases:
    try: X,Y=load(c)
    except Exception: continue
    m=(X>2)&(X<=3)
    if not m.any(): continue
    X=X[m]; Y=Y[m]; keep=np.array([x not in seen for x in X]); seen.update(X[keep].tolist()); X=X[keep]; Y=Y[keep]
    if len(X)==0: continue
    yM=ev(P,Q,X,UN_N,UN_D); yA=ev(list(P0),list(Q0),X,set(),set()); f=re.match(r'[a-zA-Z]+\d?',c).group(0)[:2]
    for d in (fam[f],tot): d[0]+=len(X); d[1]+=int(np.sum(yM==Y)); d[2]+=int(np.sum(yA==Y))
    Xall.append(X); Yall.append(Y)
print(f"ALL unique x in (2,3]: n={tot[0]}  set {tot[1]/tot[0]:.4f}   grid_a stored {tot[2]/tot[0]:.4f}")
for f,(n,a,b) in sorted(fam.items(),key=lambda t:-t[1][0]): print(f"   family {f:4s} n={n:8d}  set {a/n:.4f}  grid_a {b/n:.4f}")
X=np.concatenate(Xall); Y=np.concatenate(Yall); np.save('SCRATCH/allXY.npy',np.array([X,Y]))
yM=ev(P,Q,X,UN_N,UN_D); d=(Y-yM)/np.where(Y!=0,np.nextafter(np.abs(Y),np.inf)-np.abs(Y),1); bins=np.linspace(2,3,21)
print("match by x: "+" ".join(f"{np.mean(yM[(X>=lo)&(X<hi)]==Y[(X>=lo)&(X<hi)]):.3f}" for lo,hi in zip(bins,bins[1:])))
print("mean(Excel-sim) ulp by x: "+" ".join(f"{d[(X>=lo)&(X<hi)].mean():+.3f}" for lo,hi in zip(bins,bins[1:])))
