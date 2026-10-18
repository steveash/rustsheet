# feasible range of each coefficient delta under all constraints (pure LP, all constraints hard)
import sys, numpy as np
from scipy.optimize import linprog
fn=sys.argv[1]; deg=int(sys.argv[2]); R=float(sys.argv[3]); marg=float(sys.argv[4]) if len(sys.argv)>4 else 0.0
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    lo=float(a[2]); hi=float(a[3])
    if lo>-R or hi<R: Z.append(float.fromhex(a[1])); LO.append(lo+marg); HI.append(hi-marg)
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI)
A=np.vstack([(Z/0.25)**j for j in range(deg+1)]).T
Aub=np.vstack([A,-A]); bub=np.concatenate([HI,-LO])
res=[]
for j in range(deg+1):
    for s in (1,-1):
        c=np.zeros(deg+1); c[j]=s
        r=linprog(c,A_ub=Aub,b_ub=bub,bounds=[(None,None)]*(deg+1),method='highs')
        res.append(s*r.fun if r.status==0 else None)
        if r.status!=0: print('status',r.status,r.message); 
print('n',len(Z),'ranges (min,max) per d_j (units u2/0.25^j):',[(res[2*j],res[2*j+1]) for j in range(deg+1)])
# center (Chebyshev-like): maximize margin t
c=np.zeros(deg+2); c[-1]=-1
Aub2=np.hstack([Aub,np.ones((len(bub),1))])
r=linprog(c,A_ub=Aub2,b_ub=bub,bounds=[(None,None)]*(deg+1)+[(None,10)],method='highs')
print('max-margin',r.x[-1] if r.status==0 else r.message,'center',r.x[:-1] if r.status==0 else None)
