# Exhaustive per-run sweep of head offsets. Within one run of consecutive doubles (width ~2e-12 relative) any
# coefficient change acts on every stage as a CONSTANT pre-rounding offset, so the set of all head polynomials with
# grid_a's op structure is parameterised per run by (c1,c2,c3 at the sums s1..s3, c4 at the hand-off sum s4).
# c1 is dropped (1 ulp(s1) = 0.001 u at s4). For each (c2,c3) on a grid the best c4 is found exactly by interval
# coverage: point i is reproduced iff round(v4'+c4*u) in {s4+m*u : m admissible}, a union of unit intervals in c4.
# Then the same for D's head (g2 at d2, g3 continuous at the hand-off d3) with N held at its best, then N again.
import sys, numpy as np, time
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def head(c,x,offs):   # offs: dict stage->offset in ulp(s_stage); returns v_K pre-rounding (LD) for K=max stage
    s=LD(c[0])+0*x; K=max(offs)
    for i in range(1,K+1):
        v=sd(s*x)+LD(c[i])
        if i in offs: v=v+offs[i]*ulp(sd(v))
        if i==K: return v
        s=sd(v)
def best_offset(v,s_ref,u,adm,ms):
    # coverage sweep over c (in u): intervals [ (s_ref+m u - v)/u - 0.5, +1 ) for admissible m
    base=np.array((s_ref-v)/u,dtype=np.float64)
    lo=np.concatenate([base[adm[m]]+m-0.5 for m in ms]); 
    if len(lo)==0: return 0,0.0
    ev=np.concatenate([lo,lo+1.0]); sg=np.concatenate([np.ones(len(lo)),-np.ones(len(lo))])
    o=np.lexsort((sg,ev)); cs=np.cumsum(sg[o]); k=int(np.argmax(cs)); return int(cs[k]), float(ev[o][k])  # ends before starts at ties: half-open intervals
ms=list(range(-3,4))
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    sN=LD(P0[0])+0*X
    for i in range(1,5): sN=sd(sd(sN*X)+LD(P0[i]))
    s4=sN; u=ulp(s4)
    dD=LD(Q0[0])+0*X
    for i in range(1,4): dD=sd(sd(dD*X)+LD(Q0[i]))
    d3=dD; uD=ulp(d3)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    base=np.mean(yof(s4,d3)==Y)
    print(f"== {case} x0={float(X[n//2]):.4f} n={n} stored grid_a exact {base:.3f}")
    print(f"   point-dependent span bound at v4 from head re-decisions (u): P4 {float(ulp(sd(s4*0+sd(LD(P0[3])*X)))[0]/u[0]):.3f}  + x*ulp(s3) {float(X[0]*ulp(sd(LD(P0[3])+0*X))[0]/u[0]):.3f}  + x*ulp(P3) {float(X[0]*ulp(sd(LD(P0[2])*X))[0]/u[0]):.3f}  + x^2*ulp(s2) {float(X[0]**2*ulp(sd(LD(P0[2])+0*X))[0]/u[0]):.3f}")
    curN=(0.0,0.0,0.0); curD=(0.0,0.0); s4cur=s4; d3cur=d3
    for rnd in range(3):
        # ---- N sweep with D fixed at d3cur
        admN={m: yof(sd(s4+m*u),d3cur)==Y for m in ms}
        t0=time.time(); best=(-1,None); grid2=np.arange(-8,8)/8.0; grid3=np.arange(-128,128)/32.0
        for c2 in grid2:
            for c3 in grid3:
                v4=head(P0,X,{2:c2,3:c3,4:0.0}) if (c2 or c3) else head(P0,X,{4:0.0})
                cnt,c4=best_offset(v4,s4,u,admN,ms)
                if cnt>best[0]: best=(cnt,(c2,c3,c4),v4)
        cnt0,c40=best_offset(head(P0,X,{4:0.0}),s4,u,admN,ms)
        print(f"   round {rnd} N: c4-only best {cnt0/n:.3f} at c4={c40:+.3f}; (c2,c3,c4) best {best[0]/n:.3f} at c2={best[1][0]:+.3f} c3={best[1][1]:+.3f} c4={best[1][2]:+.3f}  [{time.time()-t0:.0f}s, {len(grid2)*len(grid3)} combos]")
        curN=best[1]; s4cur=sd(best[2]+(curN[2]+1/64)*u)   # coverage cells are half-open [e,next): realise just inside
        print(f"   round {rnd} N: direct simulation of that best = {np.mean(yof(s4cur,d3cur)==Y):.3f}")
        # ---- D sweep with N fixed at s4cur
        admD={m: yof(s4cur,sd(d3+m*uD))==Y for m in ms}
        t0=time.time(); bestD=(-1,None); gridg2=np.arange(-16,16)/16.0
        for g2 in gridg2:
            v3=head(Q0,X,{2:g2,3:0.0}) if g2 else head(Q0,X,{3:0.0})
            cnt,g3=best_offset(v3,d3,uD,admD,ms)
            if cnt>bestD[0]: bestD=(cnt,(g2,g3),v3)
        cnt0,g30=best_offset(head(Q0,X,{3:0.0}),d3,uD,admD,ms)
        print(f"   round {rnd} D: g3-only best {cnt0/n:.3f} at g3={g30:+.3f}; (g2,g3) best {bestD[0]/n:.3f} at g2={bestD[1][0]:+.3f} g3={bestD[1][1]:+.3f}  [{time.time()-t0:.0f}s]")
        curD=bestD[1]; d3cur=sd(bestD[2]+(curD[1]+1/64)*uD)
        print(f"   round {rnd} D: direct simulation of that best = {np.mean(yof(s4cur,d3cur)==Y):.3f}")
        print(f"   round {rnd} joint exact {np.mean(yof(s4cur,d3cur)==Y):.3f}")
