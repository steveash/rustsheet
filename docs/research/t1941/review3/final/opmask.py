# Op-level store/extended masks around the hand-off (N ops P3,s3,P4,s4,P5,s5; D ops P2,d2,P3,d3,P4,d4; 1=stored, 0=kept 64-bit), scored on clusters.
import sys, numpy as np, itertools
sys.path.insert(0,'docs/research/t1941/review3')
from spec import load, p as P0, q as Q0
LD=np.longdouble
def sd(v): return v.astype(np.float64).astype(LD)
clus=[]
for nn in range(1,17):
    for h in 'ab':
        X0,Y0=load('q2%02d%s'%(nn,h)); br=[0]+[i+1 for i in range(len(X0)-1) if np.nextafter(X0[i],np.inf)!=X0[i+1]]+[len(X0)]
        clus+=[(X0[a:b],Y0[a:b]) for a,b in zip(br,br[1:]) if b-a>=256]
XC=np.concatenate([X for X,Y in clus]).astype(LD); YC=np.concatenate([Y for X,Y in clus]); TC=sd(XC-LD(2))
PQ=np.load('SCRATCH/cand9_PQ.npy'); P=[float(v) for v in PQ[0]]; Q=[float(v) for v in PQ[1][:9]]
def m(y): return float(np.mean(np.asarray(y,dtype=np.float64)==YC))
def hm(c,x,mask):
    s=LD(c[0])+0*x
    for i in range(1,len(c)):
        p=s*x
        if mask.get('P%d'%i,1): p=sd(p)
        s=p+LD(c[i])
        if mask.get('s%d'%i,1): s=sd(s)
    return s
opsN=['P3','s3','P4','s4','P5','s5']; opsD=['P2','s2','P3','s3','P4','s4']
D=hm(Q,XC,{}); res=[]
for bits in itertools.product((0,1),repeat=6): res.append((m(sd(TC*sd(hm(P,XC,dict(zip(opsN,bits)))/D))),bits))
res.sort(reverse=True); print("N masks:",res[:8]); bestN=dict(zip(opsN,res[0][1])); N=hm(P,XC,bestN); res=[]
for bits in itertools.product((0,1),repeat=6): res.append((m(sd(TC*sd(N/hm(Q,XC,dict(zip(opsD,bits)))))),bits))
res.sort(reverse=True); print("D masks:",res[:8])
