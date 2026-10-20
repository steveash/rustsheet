# Joint per-run fit of the "grid_a op structure, any coefficients with late compensation" class, now including the
# parity-flipping hand-off rule: offsets c2 (ulp s2), c3 (ulp s3) and (f4, tie parity) at s4; g2 (ulp d2) and (g3, parity)
# at d3. Integer parts at s2/s3 (d2) leak into v4 (v3) as sub-u constants, which the hand-off offset absorbs.
import sys, numpy as np, time
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def rn(v,g,odd):
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
def chain(c,x,offs,hand,K):
    # offs: {stage: offset in ulp(s_stage)} applied before RNE; hand: (f,odd) parity rule at the hand-off stage K
    s=LD(c[0])+0*x
    for i in range(1,K+1):
        v=sd(s*x)+LD(c[i])
        if i in offs: v=v+offs[i]*ulp(sd(v))
        if i==K: g=ulp(sd(v)); s=rn(v+hand[0]*g,g,hand[1])
        else: s=sd(v)
    return s
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
C2=np.arange(-2,3)/4.0; C3=np.arange(-8,9)/8.0; FP=[(f,o) for f in np.arange(-4,4)/8.0 for o in (False,True)]
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    s4=chain(P0,X,{},(0.0,False),4); d3=chain(Q0,X,{},(0.0,False),3); base=np.mean(yof(s4,d3)==Y)
    t0=time.time(); curD=d3; bestN=(base,None,s4); bestD=(base,None,d3)
    for rnd in range(2):
        # N joint over (c2,c3,f4,par) with D fixed
        res=[]
        for c2 in C2:
            for c3 in C3:
                for fp in FP:
                    s4v=chain(P0,X,{2:c2,3:c3},fp,4); res.append((np.mean(yof(s4v,curD)==Y),(c2,c3)+fp,s4v))
        bestN=max(res,key=lambda r:r[0]); curN=bestN[2]
        # hand-off-only reference
        ref=max((np.mean(yof(chain(P0,X,{},fp,4),curD)==Y),fp) for fp in FP)
        res=[]
        for g2 in C2:
            for fp in FP:
                d3v=chain(Q0,X,{2:g2},fp,3); res.append((np.mean(yof(curN,d3v)==Y),(g2,)+fp,d3v))
        bestD=max(res,key=lambda r:r[0]); curD=bestD[2]
        print(f"== {case} x={float(X[n//2]):.4f} n={n} round {rnd}: stored {base:.3f}; N hand-off-only {ref[0]:.3f} {ref[1]}; N joint {bestN[0]:.3f} at c2={bestN[1][0]:+.2f} c3={bestN[1][1]:+.3f} f4={bestN[1][2]:+.3f} ties->{'odd' if bestN[1][3] else 'even'}; +D {bestD[0]:.3f} at g2={bestD[1][0]:+.2f} g3={bestD[1][1]:+.3f} ties->{'odd' if bestD[1][2] else 'even'}  [{time.time()-t0:.0f}s]", flush=True)
