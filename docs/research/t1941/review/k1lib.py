# T-1941 review (Fable 5.1). K1 helpers: exact x87 simulation variants of a raw-y Horner N/D, bin-mean residuals, per-stage blame.
import numpy as np, sys, re
L=np.longdouble
def st(v): return np.asarray(v,dtype=L).astype(np.float64)   # RN53(RN64(.)): numpy longdouble ops are RN64
def rn64(v): return np.asarray(v,dtype=L)
def load_cand(path):
    N=D=None
    for l in open(path):
        if l.startswith('N '): N=[float.fromhex(v) for v in l.split()[1:]]
        if l.startswith('D '): D=[float.fromhex(v) for v in l.split()[1:]]
    return N,D
def load_rows(path):
    X=[];G=[]
    for l in open(path):
        p=l.split(); X.append(float.fromhex(p[0])); G.append(float.fromhex(p[1]))
    return np.array(X),np.array(G)
def horner(c,y,scheme='stored',flip=None):
    """scheme: 'stored' = product stored, sum stored; 'psum' = product RN64 only, sum stored;
    'reg' = everything RN64, final stored; flip=(stage_index,kind,delta) nudges one stage by delta ulps."""
    s=np.full_like(y,c[-1]) if scheme!='reg' else np.full(y.shape,L(c[-1]),dtype=L)
    k=0
    for ci in c[-2::-1]:
        if scheme=='stored':
            p=st(L(s)*L(y)); 
            if flip and flip[0]==k and flip[1]=='p': p=p+flip[2]*np.spacing(p)
            s=st(L(p)+L(ci))
            if flip and flip[0]==k and flip[1]=='s': s=s+flip[2]*np.spacing(s)
        elif scheme=='psum':
            p=rn64(L(s)*L(y)); s=st(p+L(ci))
        elif scheme=='reg':
            s=rn64(rn64(s*L(y))+L(ci))
        k+=1
    return s if scheme!='reg' else s
def G(N,D,y,scheme='stored',flipN=None,flipD=None,divflip=0):
    n=horner(N,y,scheme,flipN); d=horner(D,y,scheme,flipD)
    g=st(L(n)/L(d))
    if divflip: g=g+divflip*np.spacing(g)
    return g
