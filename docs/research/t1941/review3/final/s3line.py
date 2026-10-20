# Decisive line test: does Excel's residual contain the s3 (and D.d2) rounding line? Stored-s3 sim has it; exact-s3 sim does not.
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, exact
from fractions import Fraction as F
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(v.astype(np.float64)); return (np.nextafter(a,np.inf)-a).astype(LD)
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
def amp(r,f): k=np.arange(len(r)); return np.dot(r-r.mean(),np.exp(-2j*np.pi*f*k))
def fline(c,x,k,prod):
    S=F(c[0]); dS=F(0)
    for i in range(1,k+(0 if prod else 1)):
        Pp=S*x; dP=dS*x+S; S=Pp+F(c[i]); dS=dP
    if prod: Pp=S*x; dP=dS*x+S; S=Pp; dS=dP
    u=F(2)**(math.floor(math.log2(abs(float(S))))-52); f=float((dS*F(2)**-51/u)%1); return f-1 if f>0.5 else f
def hm(c,x,unst):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i])
        if i not in unst: s=sd(s)
    return s
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a","e501","e503","e511","r204","e403","e404","e405"]
hdr=[('N','s',3),('N','P',4),('N','s',4),('D','d',2),('D','P',3)]
print("line amplitudes: |E| = Excel-exact, |S| = stored sim, |M| = M* sim (all /sqrt(n)); phase of E vs S in cycles.")
print("run     x        "+"   ".join(f"{a}.{b}{k}: |E| |S| |M| phE-S" for a,b,k in hdr))
for case in RUNS:
    X0,Y0=load(case); m=(X0>2)&(X0<=3); X0=X0[m]; Y0=Y0[m]
    br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
    a,b=max(zip(br,br[1:]),key=lambda t:t[1]-t[0]); b=min(b,a+8192)
    X=X0[a:b].astype(LD); Y=Y0[a:b]; t=sd(X-LD(2)); n=b-a; ex=exact(X0[a:b]); uy=ulp(Y.astype(LD))
    def res(y): return np.array((y.astype(LD)-ex)/uy,dtype=np.float64)
    rE=res(Y); rS=res(sd(t*sd(hm(P,X,set())/hm(Q,X,set())))); rM=res(sd(t*sd(hm(P,X,{3})/hm(Q,X,{2})))); x0=F(X0[a+n//2]); cells=[]
    for side,kind,k in hdr:
        c=P if side=='N' else Q; f=fline(c,x0,k,kind=='P'); AE,AS,AM=amp(rE,f),amp(rS,f),amp(rM,f)
        cells.append(f"{abs(AE)/np.sqrt(n):5.2f} {abs(AS)/np.sqrt(n):5.2f} {abs(AM)/np.sqrt(n):5.2f} {np.angle(AE/AS)/(2*np.pi):+.2f}")
    print(f"{case:6s} {float(X0[a]):.5f}  "+"   ".join(cells),flush=True)
