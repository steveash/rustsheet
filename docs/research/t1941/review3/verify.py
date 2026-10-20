# Verification of the phase-measured head offsets (NEGATIVE result, kept for the record): move p1..p3 (q0..q2) by the
# measured integer ulps, re-solve the remaining coefficients by least squares on [2,3] so N (D) is unchanged, round to
# doubles, scan the hand-off constant's integer part, simulate the full stored chain and score. Exact rates 0.20-0.57.
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulpf(c): return 2.0**(math.floor(math.log2(abs(c)))-52)
def chain(c,x):
    s=LD(c[0])+0*x
    for i in range(1,len(c)): s=sd(sd(s*x)+LD(c[i]))
    return s
def ysim(p,q,X): t=sd(X-LD(2)); return sd(t*sd(chain(p,X)/chain(q,X))).astype(np.float64)
xs=np.linspace(2,3,2001)
def compensate(c,head_delta,fix_const=None):
    n=len(c)-1; K=max(head_delta)+1
    H=sum(head_delta[j]*ulpf(c[j])*xs**(n-j) for j in head_delta)
    cols=[xs**(n-j) for j in range(K,n+1)]
    if fix_const is not None:
        H2=H+fix_const*xs**(n-K); A=np.array(cols[1:]).T; sol,res,_,_=np.linalg.lstsq(A,-H2,rcond=None); d=np.concatenate([[fix_const],sol])
    else:
        A=np.array(cols).T; d,res,_,_=np.linalg.lstsq(A,-H,rcond=None)
    newc=list(c)
    for j in head_delta: newc[j]=c[j]+head_delta[j]*ulpf(c[j])
    for i,j in enumerate(range(K,n+1)): newc[j]=c[j]+d[i]
    resid=H+sum(d[i]*xs**(n-j) for i,j in enumerate(range(K,n+1)))
    return [float(np.float64(v)) for v in newc], d, resid
u=2.0**-54
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
data={}
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); data[case]=(X0[a:b].astype(LD),Y0[a:b])
def score(p,q):
    r={c:np.mean(ysim(p,q,X)==Y) for c,(X,Y) in data.items()}; return r, np.mean(list(r.values()))
base,bm=score(P0,Q0); print("grid_a: "+" ".join(f"{c}:{v:.3f}" for c,v in base.items())+f" | mean {bm:.3f}")
headN={1:1693013,2:-124864,3:27519}; headD={0:-873813,1:69888,2:-8340}
pN,dN,rN=compensate(P0,headN); qD,dD,rD=compensate(Q0,headD)
print(f"N free LS: Delta p4 = {dN[0]/u:+.1f} u, late deltas (ulps of each) = "+" ".join(f"p{j}:{dN[i]/ulpf(P0[j]):+.0f}" for i,j in enumerate(range(4,10)))+f"; max |function residual| = {np.max(np.abs(rN))/u:.3f} u")
print(f"D free LS: Delta q3 = {dD[0]/u:+.1f} u, late deltas = "+" ".join(f"q{j}:{dD[i]/ulpf(Q0[j]):+.0f}" for i,j in enumerate(range(3,9)))+f"; max |function residual| = {np.max(np.abs(rD))/u:.3f} u")
def offset_at(c0,c1,x,K): return sum((c1[j]-c0[j])*x**(K-j) for j in range(0,K+1))/u
results=[]
for lab,c0,head,K,target in (("N",P0,headN,4,-0.116),("D",Q0,headD,3,0.463)):
    cands=[]; free,dfree,_=compensate(c0,head); base_off=offset_at(c0,free,2.25,K)
    for m in range(-3,4):
        want=target+round(base_off-target)+m; shift=(want-base_off)*u
        cc,d,res=compensate(c0,head,fix_const=dfree[0]+shift); cands.append((m,cc,offset_at(c0,cc,2.25,K),np.max(np.abs(res))/u))
    results.append((lab,cands))
for m,cc,off,res in results[0][1]:
    r,mean=score(cc,Q0); print(f"N candidate m={m:+d}: offset(2.25)={off:+.3f} u, func resid {res:.3f} u | mean {mean:.3f}")
for m,cc,off,res in results[1][1]:
    r,mean=score(P0,cc); print(f"D candidate m={m:+d}: offset(2.25)={off:+.3f} u, func resid {res:.3f} u | mean {mean:.3f}")
