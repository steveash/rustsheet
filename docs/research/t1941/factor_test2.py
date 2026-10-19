# T-1941: impose N(2)=0 (N = (x-2)*P) by refitting P against the candidate's A on [0.7,1.5], D fixed; compare forms.
import numpy as np, sys, mpmath as mp
sys.path.insert(0,'.')
from exsearch import dataset
mp.mp.dps=40
exec(open('../factor_test.py').read().split('x,b=dataset')[0])
d=load('../near1_candidate_w1.txt')
x,b=dataset(1024); t=st(L(x)-L(1)); u=st(L(x)-L(2)); near=np.abs(t)<0.05
for a,q in [('N2','D2'),('N3','D3')]:
    N=[mp.mpf(v) for v in d[a]]; D=d[q]
    xs=[mp.mpf(0.7)+mp.mpf(0.8)*(1-mp.cos(mp.pi*(i+0.5)/200))/2 for i in range(200)]
    pv=lambda c,z: mp.fsum(ci*z**k for k,ci in enumerate(c))
    # min sum ((z-2)P(z) - N(z))^2 / D(z)^2 weighted  (relative in A)
    M=mp.matrix(len(xs),9); rhs=mp.matrix(len(xs),1)
    for i,z in enumerate(xs):
        w=1/(pv(N,z))  # relative error in N
        for k in range(9): M[i,k]=(z-2)*z**k*w
        rhs[i]=1
    P=mp.lu_solve(M.T*M, M.T*rhs)
    P=[float(P[k]) for k in range(9)]
    Nf=[0.0]*10
    for k in range(10): Nf[k]=float((mp.mpf(P[k-1]) if k>=1 else 0)-2*(mp.mpf(P[k]) if k<9 else 0))
    hD=horner(D,x); hP=horner(P,x); hNf=horner(Nf,x); hN=horner(d[a],x)
    forms={'orig N/D':st(L(t)*L(st(L(hN)/L(hD)))),
      'Nf=(x-2)P expanded, N/D':st(L(t)*L(st(L(hNf)/L(hD)))),
      'fl(u*P)/D':st(L(t)*L(st(L(st(L(u)*L(hP)))/L(hD)))),
      't*u*(P/D)':st(L(st(L(t)*L(u)))*L(st(L(hP)/L(hD)))),
      '(t*(P/D))*u':st(L(st(L(t)*L(st(L(hP)/L(hD)))))*L(u)),
      't*(u*(P/D))':st(L(t)*L(st(L(u)*L(st(L(hP)/L(hD)))))),
      '(t*u*P)/D':st(L(st(L(st(L(t)*L(u)))*L(hP)))/L(hD))}
    for k,v in forms.items():
        m=(v==b); e=np.abs(v-b)/np.spacing(np.abs(b)); print(f'{a}/{q} {k:26s} all {m.mean():.4f} near1 {m[near].mean():.4f} far {m[~near].mean():.4f} med|ulp| {np.median(e):.1f}')
