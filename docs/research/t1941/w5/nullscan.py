# Local common-factor test: move c5 along its 4 sloppiest function-preserving directions (SVD of the relative
# sensitivity matrix) in fine steps and score exactly. A hidden common-factor offset would show as a peak.
import numpy as np
from sc import candidate, score_batch
from metrics import XF, BF, XN, BN
N=candidate('N3'); D=candidate('D3')
xs=np.linspace(0.7,1.5,200).astype(np.longdouble)
Nv=np.polyval(N[::-1].astype(np.longdouble),xs); Dv=np.polyval(D[::-1].astype(np.longdouble),xs)
cols=[N[i]*xs**i/Nv for i in range(10)]+[-D[i]*xs**i/Dv for i in range(1,10)]
U,s,Vt=np.linalg.svd(np.array(cols,dtype=float).T,full_matrices=False)
X=np.r_[XF[:20000],XN[:20000]]; B=np.r_[BF[:20000],BN[:20000]]
for k in [18,17,16,15]:
    for span,nstep in [(1e-10,2001),(1e-8,2001)]:
        eps=np.linspace(-span,span,nstep)
        P=np.array([N*(1+e*Vt[k][:10]) for e in eps]); Q=np.array([np.r_[D[0],D[1:]*(1+e*Vt[k][10:])] for e in eps])
        sf=score_batch(X[:20000],B[:20000],P,Q); sn=score_batch(X[20000:],B[20000:],P,Q)
        i=np.argmax(sf+sn); j=nstep//2
        print('dir %d span %.0e: c5 far %.4f near %.4f | best far %.4f near %.4f at eps %.2e | median far %.4f near %.4f'%(k,span,sf[j],sn[j],sf[i],sn[i],eps[i],np.median(sf),np.median(sn)),flush=True)
