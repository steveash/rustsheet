# T-1941 W6: positive control for polytest.py. Truth = Clenshaw / Horner-z with Chebyshev coefficients from Chebyshev
# EXTREMA nodes (degree K); scored with the polytest coefficients (Chebyshev ROOT nodes). Coherence should be high.
import sys, numpy as np, mpmath as mp, pickle
sys.path.insert(0,sys.path[0]); import polytest as P
from lib6 import st, L
X,T,E=P.X,P.T,P.E; Xa=P.Xa
def cheb_ext(K):
    nodes=[mp.cos(mp.pi*k/K) for k in range(K+1)]; f=[P.Rt((z+7)/4) for z in nodes]
    a=[]
    for j in range(K+1):
        s=sum((f[k]/2 if k in (0,K) else f[k])*mp.cos(j*mp.pi*k/K) for k in range(K+1))*2/K
        a.append(s/2 if j==K else s)
    return a
z=st(L(4.0)*L(Xa)-L(7.0))
for K in (14,16):
    at=cheb_ext(K); a=P.cheb(K)
    print('K',K,'coef rel diff max %.1e'%max(float(abs(at[j]/a[j]-1)) for j in range(K)))
    for form,tr,cand in (('clenshaw',P.clenshaw(at,z),P.clenshaw(a,z)),('horner z',P.hornerv(P.cheb_to_mono(at,K),z),P.hornerv(P.cheb_to_mono(a,K),z))):
        tr=tr.reshape(X.shape); out=[]
        for i,(hi,lo,u) in enumerate(T):
            e=((tr[i]-hi)-lo)/u; F=np.abs(np.fft.rfft(e-e.mean()))[3:257]; P.Ez[i]=(F-F.mean())/F.std()
        print('  %s: truth-vs-other-coefs %.3f, exact R match %.3f'%(form,P.score(cand),np.mean(tr.reshape(-1)==cand)))
