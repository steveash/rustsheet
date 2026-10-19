# T-1941 W6: exact x87-stored simulators for the near-1 (K3) and (x-2) (K2) GAMMALN kernels, and row loading.
import numpy as np, os
L=np.longdouble
HERE=os.path.dirname(os.path.abspath(__file__))
def st(v): return np.asarray(v,dtype=L).astype(np.float64)
def horner(c_desc,x):
    s=np.full_like(x,c_desc[0])
    for ci in c_desc[1:]: s=st(L(st(L(s)*L(x)))+L(ci))
    return s
def read_k3(path=os.path.join(HERE,'..','w5','p4','ss_fc.txt')):
    ls=[l.split() for l in open(path) if l.strip() and not l.startswith('#')]
    return [float.fromhex(v) for v in ls[0]],[float.fromhex(v) for v in ls[1]]   # ascending
def read_k2(path=os.path.join(HERE,'..','w2','efC.txt')):
    P=Q=None
    for l in open(path):
        p=l.split()
        if p and p[0]=='p': P=[float.fromhex(v) for v in p[1:]]
        if p and p[0]=='q': Q=[float.fromhex(v) for v in p[1:]]
    return P,Q   # highest first
def k3(x,N,D,one=1.0):
    A=st(L(horner(N[::-1],x))/L(horner(D[::-1],x))); t=st(L(x)-L(one)); return st(L(t)*L(A)),A
def k2(x,P,Q,two=2.0):
    S=st(L(horner(P,x))/L(horner(Q,x))); m=st(L(x)-L(two)); return st(L(m)*L(S)),S
def load(path,lo=-1e308,hi=1e308):
    X=[];B=[];C=[]
    for l in open(path):
        p=l.split(); x=float.fromhex(p[0])
        if lo<=x<hi: X.append(x); B.append(float.fromhex(p[1])); C.append(p[2]+(':'+p[3] if len(p)>3 else ''))
    X=np.array(X); o=np.argsort(X,kind='stable')
    return X[o],np.array(B)[o],np.array(C)[o]
