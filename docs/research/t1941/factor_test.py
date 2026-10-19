# T-1941: does Excel evaluate the near-1 kernel with the (x-2) zero factored out? Compare evaluation orders.
import numpy as np, sys
sys.path.insert(0,'.')
from exsearch import dataset
L=np.longdouble
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
def horner(c,x):
    s=np.full_like(x,c[-1])
    for ci in c[-2::-1]: s=st(st(L(s)*L(x))+L(ci)) if False else st(L(st(L(s)*L(x)))+L(ci))
    return s
def load(f):
    d={}
    for l in open(f):
        if l.startswith('#') or not l.strip(): continue
        p=l.split(); d[p[0]]=[float.fromhex(v) for v in p[2:]]
    return d
def deflate(n,r):
    deg=len(n)-1; P=[0.0]*deg; P[deg-1]=n[deg]
    for k in range(deg-1,0,-1): P[k-1]=n[k]+r*P[k]
    return P
x,b=dataset(1024); t=st(L(x)-L(1)); u=st(L(x)-L(2))
d=load('../near1_candidate_w1.txt')
for a,q in [('N2','D2'),('N3','D3')]:
    N=d[a]; D=d[q]; P=deflate(N,2.0)
    hN=horner(N,x); hD=horner(D,x); hP=horner(P,x)
    forms={
     'A=N/D, t*A':            st(L(t)*L(st(L(hN)/L(hD)))),
     'A=fl(u*P)/D, t*A':      st(L(t)*L(st(L(st(L(u)*L(hP)))/L(hD)))),
     't*u*(P/D)':             st(L(st(L(t)*L(u)))*L(st(L(hP)/L(hD)))),
     '(t*(P/D))*u':           st(L(st(L(t)*L(st(L(hP)/L(hD)))))*L(u)),
     't*(u*(P/D))':           st(L(t)*L(st(L(u)*L(st(L(hP)/L(hD)))))),
     '(t*u*P)/D':             st(L(st(L(st(L(t)*L(u)))*L(hP)))/L(hD)),
    }
    near=np.abs(t)<0.05
    for k,v in forms.items():
        m=(v==b); print(f'{a}/{q} {k:22s} all {m.mean():.4f} near1 {m[near].mean():.4f} far {m[~near].mean():.4f}')
print(x[:3],u[:3],hN[:3],hP[:3],(u*hP)[:3],hD[:3],b[:3])
v1=forms['A=N/D, t*A']; v2=forms['A=fl(u*P)/D, t*A']
print(v1[:3],v2[:3],b[:3], v1.dtype, v2.dtype, len(P), P)
print((v2==b).mean(), np.mean(np.abs(v2-b)/np.spacing(np.abs(b))<=1), np.median(np.abs(v2-b)/np.spacing(np.abs(b))))
