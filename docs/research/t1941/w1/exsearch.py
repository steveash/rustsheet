import numpy as np
from csim import simA
from load2 import load_runs
def dataset(maxper=1024, seed=0):
    R=load_runs(); rng=np.random.default_rng(seed); X=[];B=[]
    for k,a in R.items():
        x=a[:,0]; b=a[:,1]
        sel=(x>=0.7)&(x<=1.5)&np.isfinite(b)
        if sel.sum()==0: continue
        idx=np.where(sel)[0]
        if len(idx)>maxper: idx=rng.choice(idx,maxper,replace=False)
        X.append(x[idx]); B.append(b[idx])
    X=np.concatenate(X); B=np.concatenate(B); o=np.argsort(X)
    return X[o],B[o]
def res_of(x,A):            # res = fl((x-1)*A); (x-0.5)-0.5 == x-1 exactly here
    t=(x-0.5)-0.5
    return (np.longdouble(t)*np.longdouble(A)).astype(np.float64)
def score(x,b,N,D):
    return np.mean(res_of(x,simA(x,N,D))==b)
