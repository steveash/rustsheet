# T-1941 review2: model-free composition test across GAMMALN pieces (W6's reachability method, extended to nested
# two-factor forms). For each window, the fraction of Excel's B values reachable as
#   m*S      : fl(m*S), m = x-2 exact, S any double          (W2's K2 form)
#   t*A      : fl(t*A), t = x-1 exact                        (K3 form)
#   t*(m*R)  : fl(t*fl(m*R))                                 (W6's (1.5,2] form)
#   m*(t*R)  : fl(m*fl(t*R))
#   (tm)*R   : fl(fl(t*m)*R)
# and the same for B's +-1,+-2 ulp neighbours (base rate of reachable doubles). A form at ~1.0 against a base rate
# well below 1 is the composition; a form at its base rate is excluded.
import numpy as np, lib2, sys
from lib2 import L,st
X,B,C,cs=lib2.ln_rows()
def reach1(f,b):
    """exists double S with fl(f*S)==b ? (search S near b/f)"""
    q=b/f; ok=np.zeros(len(b),bool); cand=[]
    for d in range(-3,4):
        Sv=q.copy()
        for _ in range(abs(d)): Sv=np.nextafter(Sv,np.inf if d>0 else -np.inf)
        hit=(st(L(f)*L(Sv))==b); ok|=hit; cand.append((hit,Sv))
    return ok,cand
def reach2(fo,fi,b):
    """exists doubles A,R with fl(fo*A)==b and A==fl(fi*R) ?"""
    ok=np.zeros(len(b),bool); oko,cand=reach1(fo,b)
    for hit,A in cand:
        oki,_=reach1(fi,A); ok|=(hit&oki)
    return ok
rng=np.random.default_rng(3)
for lo,hi in [(0.72,0.8),(0.9,0.98),(1.3,1.4),(1.4,1.47),(1.52,1.6),(1.7,1.8),(1.9,1.98),(2.02,2.1),(2.2,2.3),(2.5,2.6),(2.7,2.8),(2.9,3.0)]:
    s=np.where((X>=lo)&(X<hi))[0]
    if len(s)<500: print(f"[{lo},{hi}) only {len(s)} rows"); continue
    s=rng.choice(s,min(20000,len(s)),replace=False); x=X[s]; b=B[s]
    t=x-1.0; m=x-2.0; tm=st(L(t)*L(m))
    forms={'m*S':lambda bb: reach1(m,bb)[0],'t*A':lambda bb: reach1(t,bb)[0],'t*(m*R)':lambda bb: reach2(t,m,bb),
           'm*(t*R)':lambda bb: reach2(m,t,bb),'(tm)*R':lambda bb: reach1(tm,bb)[0]}
    out=[]
    for nm,f in forms.items():
        r=f(b).mean()
        nb=[np.nextafter(b,np.inf),np.nextafter(b,-np.inf),np.nextafter(np.nextafter(b,np.inf),np.inf),np.nextafter(np.nextafter(b,-np.inf),-np.inf)]
        base=np.mean([f(v).mean() for v in nb])
        out.append(f"{nm} {r:.3f}/{base:.3f}")
    print(f"[{lo:.2f},{hi:.2f}) n={len(s):5d}  "+"  ".join(out))
