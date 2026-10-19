# T-1941 W6: structure of the (1.5,2] kernel at x = 2 - k*2^-52. For each candidate wrapper and each k, the set of
# doubles R (inner stored quotient, ~ lnG/((x-1)(x-2)) or the analogue) that reproduce Excel's B. A valid wrapper
# admits an R(k) that is smooth in k (slope ~ R'(2)*h/ulp(R)).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import *
mp.mp.dps=40
X,B,C=load(sys.argv[1],1.99,2.0); u=2.0**-52
K=int(sys.argv[2]) if len(sys.argv)>2 else 40
byk={}
for x,b in zip(X,B): byk[round((2-x)/u)]=b
R2=float(1-mp.euler)
def win(c,n=40):
    base=c; out=[base]; a=base; d=base
    for _ in range(n): a=np.nextafter(a,np.inf); d=np.nextafter(d,-np.inf); out+= [a,d]
    return np.array(sorted(out))
def wr(name,x,R):
    t=st(L(x)-L(1.0)); m=st(L(x)-L(2.0))
    if name=='t*(m*R)': return st(L(t)*L(st(L(m)*L(R))))
    if name=='(t*m)*R': return st(L(st(L(t)*L(m)))*L(R))
    if name=='m*(t*R)': return st(L(m)*L(st(L(t)*L(R))))
    if name=='m*S':     return st(L(m)*L(R))
    if name=='t*(m*R)64': return st(L(t)*(L(m)*L(R)))
for name,Rc in (('t*(m*R)',R2),('(t*m)*R',R2),('m*(t*R)',R2),('m*S',R2),('t*(m*R)64',R2)):
    print('==',name)
    line=[]
    for k in range(1,K+1):
        if k not in byk: continue
        x=2-k*u; Rs=win(Rc if name!='m*S' else R2); ok=Rs[wr(name,np.full(len(Rs),x),Rs)==byk[k]]
        off=[int(round((r-R2)/np.spacing(R2))) for r in ok]
        line.append('%d:%s'%(k,('%d..%d'%(min(off),max(off))) if off else '-'))
    print(' '.join(line))
