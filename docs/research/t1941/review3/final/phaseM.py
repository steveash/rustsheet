# Per-stage line-phase read on long runs: Excel's pre-rounding offset at each op relative to the sim, in ulp of that op (calibrated with a planted +0.25 ulp).
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, exact
from fractions import Fraction as F
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
UN={'N':set(),'D':set()}   # stored structure
def amp(r,f): k=np.arange(len(r)); return np.dot(r-r.mean(),np.exp(-2j*np.pi*f*k))
def fline(c,x,k,prod):
    S=F(c[0]); dS=F(0)
    for i in range(1,k+(0 if prod else 1)):
        Pp=S*x; dP=dS*x+S; S=Pp+F(c[i]); dS=dP
    if prod: Pp=S*x; dP=dS*x+S; S=Pp; dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); f=float((dS*F(2)**-51/u)%1); return f-1 if f>0.5 else f
def chain_off(c,x,unst,stage,off,prod=False):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        p=s*x
        if prod and i==stage: p=p+off*ulp(sd(p))
        p=sd(p); v=p+LD(c[i])
        if (not prod) and i==stage: v=v+off*ulp(sd(v))
        s=v if i in unst else sd(v)
    return s
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
hdr=[('N','s',k) for k in (2,3,4,5,6,7)]+[('N','P',k) for k in (3,4,5)]+[('D','d',k) for k in (1,2,3,4,5)]+[('D','P',k) for k in (2,3,4)]
print("run x | per op: delta(ulp) ratio snr   "+"  ".join(f"{a}.{b}{k}" for a,b,k in hdr))
for case in RUNS:
    X0,Y0=load(case); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=b-a; ex=exact(X0[a:b]); uy=ulp(Y.astype(LD))
    def res(y): return np.array((y.astype(LD)-ex)/uy,dtype=np.float64)
    def yof(N,D): return sd(t*sd(N/D)).astype(np.float64)
    N0=chain_off(P,X,UN['N'],0,0); D0=chain_off(Q,X,UN['D'],0,0); rE=res(Y); rS=res(yof(N0,D0)); x0=F(X0[a+n//2]); cells=[]
    for side,kind,k in hdr:
        prod=(kind=='P'); c=P if side=='N' else Q; f=fline(c,x0,k,prod)
        rC=res(yof(chain_off(P,X,UN['N'],k,0.25,prod),D0)) if side=='N' else res(yof(N0,chain_off(Q,X,UN['D'],k,0.25,prod)))
        AE,AS,AC=amp(rE,f),amp(rS,f),amp(rC,f); dC=np.angle(AC/AS)/(2*np.pi); sgn=np.sign(dC) if abs(dC)>0.02 else 0.0
        snr=abs(AS)**2/n/max(np.median(np.abs(np.fft.rfft(rS-rS.mean()))**2/n),1e-12)
        cells.append(f"{sgn*np.angle(AE/AS)/(2*np.pi):+.2f} {abs(AE)/max(abs(AS),1e-9):.2f} {min(snr,999):3.0f}")
    print(f"{case:6s} {float(X0[a]):.4f}  "+"  ".join(cells),flush=True)
