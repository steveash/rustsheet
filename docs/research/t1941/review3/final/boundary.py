# Table test: runs centred on a binary fraction (2.125, 2.25, 2.5) straddle any power-of-two cell boundary; fit the hand-off rule
# below/above the centre and on quarters; controls at interior x. Uses fitrule() from drift.py.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3'); sys.path.insert(0,'SCRATCH')
from spec import load
from drift import fitrule, LD
targets=[('r903',2.125),('e801',2.125),('r901',2.25),('e804',2.25),('r901',2.5),('q221a',2.30),('r112',None),('r113',None)]
def fmt(k): return f"{k[0]:+.3f}{'o' if k[1] else 'e'}"
for case,xc in targets:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    runs=[(a,b) for a,b in zip(br,br[1:]) if b-a>=1000]
    runs=[(a,b) for a,b in runs if abs(float(X0[(a+b)//2])-xc)<1e-6] if xc is not None else [max(runs,key=lambda t:t[1]-t[0])]
    for a,b in runs[:1]:
        X=X0[a:b].astype(LD); Y=Y0[a:b]; n=b-a; mid=n//2
        lo=fitrule(X[:mid],Y[:mid]); hi=fitrule(X[mid:],Y[mid:])
        print(f"== {case} x={float(X[mid]):.6f}: below {lo[0]:.3f}->{lo[2]:.3f} {fmt(lo[1])} | above {hi[0]:.3f}->{hi[2]:.3f} {fmt(hi[1])}")
        qs=[fitrule(X[k*n//4:(k+1)*n//4],Y[k*n//4:(k+1)*n//4]) for k in range(4)]
        print("   quarters: "+" | ".join(f"{q[0]:.3f}->{q[2]:.3f} {fmt(q[1])}" for q in qs), flush=True)
