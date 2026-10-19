# T-1941 W6: model-free reachability of B under nested wrappers with an inner stored double R:
#   W1: B = fl(t*fl(m*R))   W2: B = fl(fl(t*m)*R)   W3: B = fl(m*fl(t*R))   W0: B = fl(t*A) (A free, reference)
# with t = x-1, m = x-2 exact. Rate for Excel's B against base rate for B+-1 ulp neighbours.
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import *
X,B,C=load(sys.argv[1],float(sys.argv[2]),float(sys.argv[3]))
def nb(v,d):
    for _ in range(abs(d)): v=np.nextafter(v,np.inf if d>0 else -np.inf)
    return v
def W(name,x,R):
    t=x-1.0; m=x-2.0
    if name=='t(mR)': return st(L(t)*L(st(L(m)*L(R))))
    if name=='(tm)R': return st(L(st(L(t)*L(m)))*L(R))
    if name=='m(tR)': return st(L(m)*L(st(L(t)*L(R))))
    if name=='tA':    return st(L(t)*L(R))
def reach(name,x,b):
    if name=='tA': R0=b/(x-1.0)
    else: R0=b/((x-1.0)*(x-2.0))
    ok=np.zeros(len(b),bool)
    for d in range(-4,5): ok|=(W(name,x,nb(R0,d))==b)
    return ok
w=float(sys.argv[4])
for lo in np.arange(float(sys.argv[2]),float(sys.argv[3]),w):
    s=(X>=lo)&(X<lo+w)
    if s.sum()<100: continue
    x=X[s]; b=B[s]; out=[]
    for nm in ('tA','t(mR)','(tm)R','m(tR)'):
        r=reach(nm,x,b).mean(); base=np.mean([reach(nm,x,nb(b,d)).mean() for d in (-1,1)])
        out.append('%s %.3f/%.3f'%(nm,r,base))
    print('%.3f %6d  '%(lo,s.sum())+'  '.join(out))
