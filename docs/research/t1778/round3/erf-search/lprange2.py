# ranges of d3..dn with d2 fixed (integer), in coefficient ulps; also count lattice volume estimate
import sys, numpy as np
from scipy.optimize import linprog
fn=sys.argv[1]; deg=int(sys.argv[2]); R=float(sys.argv[3]); d2=int(sys.argv[4]); upu=[float(v) for v in sys.argv[5].split(',')]  # ulps per unit for d3..
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    lo=float(a[2]); hi=float(a[3])
    if lo>-R or hi<R: Z.append(float.fromhex(a[1])); LO.append(lo-d2); HI.append(hi-d2)
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI)
A=np.vstack([(Z/0.25)**j for j in range(1,deg+1)]).T
Aub=np.vstack([A,-A]); bub=np.concatenate([HI,-LO])
for j in range(deg):
    out=[]
    for s in (1,-1):
        c=np.zeros(deg); c[j]=s
        r=linprog(c,A_ub=Aub,b_ub=bub,bounds=[(None,None)]*deg,method='highs'); out.append(s*r.fun)
    print('d%d range [%.4f, %.4f] units = %.0f ulps wide'%(j+3,out[0],out[1],(out[1]-out[0])*upu[j]))
