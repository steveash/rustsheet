# Per-run fit of the hand-off rounding rule: s4 = RN(v4 + f*u) with ties to even or to ODD (in grid_a's frame), and the
# same for D's d3 (g, tie rule). Ties-to-odd in our frame = ties-to-even on a value offset by an odd number of ulps.
import sys, numpy as np, time
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
def tail(c,k,s,x):
    for i in range(k+1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def rn(v,g,odd):   # round v to grid g; ties to even (odd=False) or to odd (odd=True)
    q=v/g; f=np.floor(q); r=q-f; fo=np.mod(f,2)==1
    up=(r>0.5)|((r==0.5)&(fo if not odd else ~fo)); return (f+up)*g
F=np.arange(-16,17)/8.0
for case in sys.argv[1:]:
    X0,Y0=load(case)
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=len(X)
    s=LD(P0[0])+0*X
    for i in range(1,4): s=sd(sd(s*X)+LD(P0[i]))
    v4=sd(s*X)+LD(P0[4]); s4=sd(v4); u=ulp(s4)
    d=LD(Q0[0])+0*X
    for i in range(1,3): d=sd(sd(d*X)+LD(Q0[i]))
    v3=sd(d*X)+LD(Q0[3]); d3=sd(v3); uD=ulp(d3)
    def yof(s4v,d3v): return sd(t*sd(tail(P0,4,s4v,X)/tail(Q0,3,d3v,X))).astype(np.float64)
    y0=yof(s4,d3); base=np.mean(y0==Y)
    t0=time.time()
    NV={(f,o):rn(v4+f*u,u,o) for f in F for o in (False,True)}
    DV={(g,o):rn(v3+g*uD,uD,o) for g in F for o in (False,True)}
    # stage 1: N alone (D stored), D alone (N stored)
    rN={k:np.mean(yof(v,d3)==Y) for k,v in NV.items()}
    rD={k:np.mean(yof(s4,v)==Y) for k,v in DV.items()}
    bN=max(rN,key=rN.get); bD=max(rD,key=rD.get)
    # stage 2: joint over the top-6 of each
    topN=sorted(rN,key=rN.get,reverse=True)[:6]; topD=sorted(rD,key=rD.get,reverse=True)[:6]
    rJ={(kn,kd):np.mean(yof(NV[kn],DV[kd])==Y) for kn in topN for kd in topD}
    bJ=max(rJ,key=rJ.get)
    fmt=lambda k:f"f={k[0]:+.3f} ties->{'odd' if k[1] else 'even'}"
    print(f"== {case} x={float(X[n//2]):.4f} n={n} stored {base:.3f} ulp(P4)/u={float(ulp(sd(s*X))[0]/u[0]):.3g}  [{time.time()-t0:.0f}s]")
    print(f"   N alone best {rN[bN]:.3f} at {fmt(bN)};  even-tie best {max(v for k,v in rN.items() if not k[1]):.3f} at f={max((k for k in rN if not k[1]),key=rN.get)[0]:+.3f};  odd-tie best {max(v for k,v in rN.items() if k[1]):.3f} at f={max((k for k in rN if k[1]),key=rN.get)[0]:+.3f}")
    print(f"   D alone best {rD[bD]:.3f} at {fmt(bD)};  even-tie best {max(v for k,v in rD.items() if not k[1]):.3f} at g={max((k for k in rD if not k[1]),key=rD.get)[0]:+.3f};  odd-tie best {max(v for k,v in rD.items() if k[1]):.3f} at g={max((k for k in rD if k[1]),key=rD.get)[0]:+.3f}")
    print(f"   joint best {rJ[bJ]:.3f} at N[{fmt(bJ[0])}] D[{fmt(bJ[1])}]")
    print("   N profile (ties->odd), f=-1..1 step 1/4: "+" ".join(f"{rN[(f,True)]:.3f}" for f in F if abs(f)<=1 and (f*4)==round(f*4)))
    print("   N profile (ties->even), f=-1..1 step 1/4: "+" ".join(f"{rN[(f,False)]:.3f}" for f in F if abs(f)<=1 and (f*4)==round(f*4)))
