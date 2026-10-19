# Move c5 along its sloppiest function-preserving directions and watch the exact score:
# if the score stays flat, the data do not pin c5's sloppy coordinates (e.g. its x=0 invariants).
import numpy as np, mpmath as mp
from sc import candidate, score
from metrics import XF, BF, XN, BN
mp.mp.dps=40
N=candidate('N3'); D=candidate('D3')
xs=np.linspace(0.7,1.5,200).astype(np.longdouble)
Nv=np.polyval(N[::-1].astype(np.longdouble),xs); Dv=np.polyval(D[::-1].astype(np.longdouble),xs)
# columns: relative change of N/D per *relative* change of each coefficient (D0 fixed)
cols=[];lab=[]
for i in range(10): cols.append(N[i]*xs**i/Nv); lab.append('N%d'%i)
for i in range(1,10): cols.append(-D[i]*xs**i/Dv); lab.append('D%d'%i)
M=np.array(cols,dtype=float).T
U,s,Vt=np.linalg.svd(M,full_matrices=False)
print('singular values',' '.join('%.1e'%v for v in s))
base=(score(XF,BF,N,D),score(XN,BN,N,D)); print('c5 far/near',base)
for k in [18,17,16,15]:
    v=Vt[k]
    for eps in [1e-14,1e-12,1e-10,1e-8,1e-6]:
        n=N.copy(); d=D.copy()
        n*= 1+eps*v[:10]; d[1:]*= 1+eps*v[10:]
        fchg=s[k]*eps
        print('dir %d (sv %.1e) eps %.0e  func change ~%.0e  far %.4f near %.4f'%(k,s[k],eps,fchg,score(XF,BF,n,d),score(XN,BN,n,d)))
