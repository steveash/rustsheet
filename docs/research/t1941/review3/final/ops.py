# Arithmetic-structure variants on clusters: stored x87 (fl53(fl64) per op), pure double, FMA Horner, extended-through-stage-K.
import sys, numpy as np, math
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
clus=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        clus+=[(X0[a:b],Y0[a:b]) for a,b in zip(br,br[1:]) if b-a>=256]
X64=np.concatenate([X for X,Y in clus]); YC=np.concatenate([Y for X,Y in clus]); XC=X64.astype(LD); TC=sd(XC-LD(2)); T64=X64-2.0
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
def m(y): return float(np.mean(np.asarray(y,dtype=np.float64)==YC))
def hx(c,x,store=True,ext_until=99):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        p=s*x; p=sd(p) if (store and i>ext_until) or (store and ext_until==99) else p
        s=p+LD(c[i]); s=sd(s) if (store and i>ext_until) or (store and ext_until==99) else s
    return s
def h64(c,x):
    s=np.full_like(x,c[0])
    for i in range(1,len(c)): s=(s*x)+c[i]
    return s
def hf(c,x):
    out=np.empty(len(x))
    for j,xv in enumerate(x):
        s=c[0]
        for i in range(1,len(c)): s=math.fma(s,xv,c[i])
        out[j]=s
    return out
print("stored x87", m(sd(TC*sd(hx(P,XC)/hx(Q,XC)))))
print("pure double", m(T64*(h64(P,X64)/h64(Q,X64))))
print("all extended", m(sd(TC*(hx(P,XC,False)/hx(Q,XC,False)))))
for K in (2,3,4,5): print(f"extended through stage {K}/{K-1}", m(sd(TC*sd(hx(P,XC,True,K)/hx(Q,XC,True,K-1)))))
print("FMA Horner", m(T64*(hf(P,X64)/hf(Q,X64))))
