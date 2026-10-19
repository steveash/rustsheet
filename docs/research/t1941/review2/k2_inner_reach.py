# T-1941 review2: is K2's stored inner S = fl(N/D) itself a product fl((x-1)*R) (one rational shared with (1.5,2])?
# Where fl((x-2)*S) = B determines S uniquely, test whether S is reachable as fl(t*R) for t = x-1 and some double
# R, against the base rate for S's +-1,+-2 ulp neighbours. Only windows where t*ulp(R) > ulp(S) discriminate.
import numpy as np, lib2
from lib2 import L,st
X,B,C,cs=lib2.ln_rows()
def cands(f,b):
    q=b/f; out=[]
    for d in range(-3,4):
        Sv=q.copy()
        for _ in range(abs(d)): Sv=np.nextafter(Sv,np.inf if d>0 else -np.inf)
        out.append((st(L(f)*L(Sv))==b,Sv))
    return out
def reach(f,b):
    ok=np.zeros(len(b),bool)
    for hit,_ in cands(f,b): ok|=hit
    return ok
rng=np.random.default_rng(7)
for lo in np.arange(2.02,3.0,0.04):
    hi=lo+0.04; s=np.where((X>=lo)&(X<hi))[0]
    if len(s)<2000: continue
    s=rng.choice(s,min(30000,len(s)),replace=False); x=X[s]; b=B[s]; m=x-2.0; t=x-1.0
    cs_=cands(m,b); nhit=sum(h for h,_ in cs_); uniq=(nhit==1)
    if uniq.sum()<500: print(f"[{lo:.2f},{hi:.2f}) S unique for {uniq.mean():.3f} of points: skip"); continue
    S=np.zeros(len(b)); 
    for h,Sv in cs_: S=np.where(h&uniq,Sv,S)
    S=S[uniq]; tt=t[uniq]
    r=reach(tt,S).mean()
    base=np.mean([reach(tt,v).mean() for v in (np.nextafter(S,np.inf),np.nextafter(S,-np.inf),np.nextafter(np.nextafter(S,np.inf),np.inf),np.nextafter(np.nextafter(S,-np.inf),-np.inf))])
    print(f"[{lo:.2f},{hi:.2f}) n={uniq.sum():5d} (unique S {uniq.mean():.2f})  S reachable as fl((x-1)*R): {r:.3f}  base {base:.3f}")
