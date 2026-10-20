import numpy as np, math, sys, gsk
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
LD=np.longdouble
d=np.load('k1R10.npz'); x,t=d['x'],d['t']; N,D=read_chain('ch1c.txt'); N=np.array(N); D=np.array(D)
ys=np.linspace(1,2,2001)
def g(deg):
    V=np.vander(ys,deg+1,increasing=True); c,*_=np.linalg.lstsq(V,1/ys,rcond=None); return c
def apply(C,j,e,deg):
    C=C.astype(LD).copy(); gk=g(deg)
    for k,v in enumerate(gk): C[j+k]+=LD(e)*LD(v)
    C[j-1]-=LD(e); return C.astype(np.float64)
eN=852*math.ulp(abs(N[1])); eD=-350*math.ulp(abs(D[2]))
best=None
for dN in (1,2,3,4,5):
    for dD in (1,2,3,4):
        N2=apply(N,2,eN,dN); D2=apply(D,3,eD,dD); ex=gsk.count(N2,D2,0,0,x,t)/len(x); print('degN %d degD %d: %.4f'%(dN,dD,ex),flush=True)
        if best is None or ex>best[0]: best=(ex,N2,D2)
open('plain0.txt','w').write("N "+" ".join(float(v).hex() for v in best[1])+"\nD "+" ".join(float(v).hex() for v in best[2])+"\n")
# then: LN=0 LDm=0 python3 mclimb.py W6DIR k1R10.npz plain0.txt plain1.txt
