# s3 precision scan (round the exact sum to B bits before the hand-off product) and "which prediction does Excel follow" tables.
import sys, numpy as np
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
def ulp(v): a=np.abs(np.asarray(v,dtype=np.float64)); return (np.nextafter(a,np.inf)-a)
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
clus=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        clus+=[(X0[a:b],Y0[a:b]) for a,b in zip(br,br[1:]) if b-a>=256]
X=np.concatenate([Xc for Xc,Yc in clus]); YC=np.concatenate([Yc for Xc,Yc in clus]); XC=X.astype(LD); TC=sd(XC-LD(2))
def roundB(v,B):
    if B>=64: return v
    m,e=np.frexp(v.astype(np.float64)); scale=np.ldexp(np.ones_like(m),e-B).astype(LD); q=v/scale
    f=np.floor(q); d=q-f; r=f+(d>0.5)+((d==0.5)&(np.mod(f,2)==1)); return r*scale
def hmB(c,x,K,B):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i]); s=roundB(s,B) if i==K else sd(s)
    return s
def hm(c,x,unst):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        s=sd(s*x)+LD(c[i])
        if i not in unst: s=sd(s)
    return s
DS=hm(Q,XC,set()); yS=sd(TC*sd(hm(P,XC,set())/DS)).astype(np.float64); yM=sd(TC*sd(hm(P,XC,{3})/hm(Q,XC,{2}))).astype(np.float64)
for B in (53,54,55,56,58,64): print(f"s3 at {B} bits: {np.mean(sd(TC*sd(hmB(P,XC,3,B)/DS)).astype(np.float64)==YC):.4f}")
def chain(c,x):
    s=LD(c[0])+0*x; out=[]
    for i in range(1,len(c)):
        pe=s*x; p=sd(pe); se=p+LD(c[i]); s=sd(se); out.append((pe,p,se,s))
    return out
cN=chain(P,XC); pe3,p3,se3,s3=cN[2]; r3=np.array((se3-s3)/ulp(s3).astype(LD),dtype=float)
pe4,p4,se4,s4=cN[3]; r4=np.array((pe4-p4)/ulp(p4).astype(LD),dtype=float)
diff=yS!=yM; fm=diff&(YC==yM); fs=diff&(YC==yS)
def pm(k): return np.sum(fm&k)/max(np.sum((fm|fs)&k),1)
print(f"differ {diff.mean():.4f}; follow M* {np.mean(YC[diff]==yM[diff]):.3f} stored {np.mean(YC[diff]==yS[diff]):.3f}")
shift=np.array(r3*ulp(s3)*X/ulp(p4),dtype=float); cross=np.abs(r4+shift)>0.5
print("P(follow M*) by margin past the boundary: "+" ".join(f"[{lo:.1f}) {pm(cross&(np.abs(r4+shift)-0.5>=lo)&(np.abs(r4+shift)-0.5<lo+0.1)):.3f}" for lo in (0,0.1,0.2,0.3,0.4,0.5)))
