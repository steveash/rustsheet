# Late-coefficient scan in whole units of each stored sum's grid (p5..p9: ulp(s_k); q4..q8: ulp(d_k)), m in [-6,6],
# scored per run by the exact rate with the (4,3)-pre offset meter on top (so per-anchor head offsets are absorbed).
import os, sys, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from spec import load, p as P0, q as Q0
import meter as M
LD=np.longdouble
def gridunit(c,i,X): S=M.sd(M.sd(M.pre(c,i,X))+LD(c[i])); return float(M.ulp(S)[0])
runs=[]
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); runs.append((case,X0[a:b].astype(LD),Y0[a:b]))
MS=range(-6,7)
for nm,c0,isq,idx in (("p",P0,False,range(5,10)),("q",Q0,True,range(4,9))):
    for i in idx:
        rows=[]
        for case,X,Y in runs:
            g=gridunit(c0,i,X); best=[]
            for m in MS:
                c=list(c0); c[i]=c[i]+m*g
                p,q=(P0,c) if isq else (c,Q0)
                base,bst,dn,dd,_,_=M.meter(X,Y,4,3,'pre',p=p,q=q,passes=2); best.append(bst)
            rows.append(best)
        rows=np.array(rows); tot=rows.mean(0); j=int(np.argmax(tot))
        perrun=[int(MS[int(np.argmax(r))]) for r in rows]
        print(f"{nm}{i}: mean best rate by m: "+" ".join(f"{m:+d}:{v:.3f}" for m,v in zip(MS,tot))+f" | overall best m={MS[j]:+d} ({tot[j]:.3f} vs m=0 {tot[6]:.3f}) | per-run best m: {perrun}", flush=True)
