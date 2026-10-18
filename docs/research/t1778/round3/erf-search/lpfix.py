# max-margin point with d2 fixed to integer(s); prints best per d2
import sys, numpy as np
from scipy.optimize import linprog
fn=sys.argv[1]; deg=int(sys.argv[2]); R=float(sys.argv[3]); d2s=[int(v) for v in sys.argv[4].split(',')]
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    lo=float(a[2]); hi=float(a[3])
    if lo>-R or hi<R: Z.append(float.fromhex(a[1])); LO.append(lo); HI.append(hi)
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI)
A=np.vstack([(Z/0.25)**j for j in range(1,deg+1)]).T
for d2 in d2s:
    Aub=np.vstack([A,-A]); bub=np.concatenate([HI-d2,-(LO-d2)])
    c=np.zeros(deg+1); c[-1]=-1
    r=linprog(c,A_ub=np.hstack([Aub,np.ones((len(bub),1))]),b_ub=bub,bounds=[(None,None)]*deg+[(None,10)],method='highs')
    print('d2',d2,'margin',r.x[-1] if r.status==0 else r.message,' '.join('%.6f'%v for v in r.x[:-1]) if r.status==0 else '')
