import sys, numpy as np
from scipy.optimize import linprog
fn=sys.argv[1]; deg=int(sys.argv[2])  # number of free coeffs beyond c2: deg -> G = d2 + d3 z + ... (deg+1 terms)
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    Z.append(float.fromhex(a[1])); LO.append(float(a[2])); HI.append(float(a[3]))
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI); n=len(Z)
A=np.vstack([(Z/0.25)**j for j in range(deg+1)]).T
# L1 slack minimization: lo - s_i <= A d <= hi + s_i, s_i>=0 (one slack per point)
from scipy.sparse import hstack, vstack, identity, csr_matrix
I=identity(n,format='csr'); Ac=csr_matrix(A)
Aub=vstack([hstack([Ac,-I]), hstack([-Ac,-I])])
bub=np.concatenate([HI-1e-3, -(LO+1e-3)])
c=np.concatenate([np.zeros(deg+1), np.ones(n)])
bounds=[(None,None)]*(deg+1)+[(0,None)]*n
r=linprog(c,A_ub=Aub,b_ub=bub,bounds=bounds,method='highs')
d=r.x[:deg+1]; v=A@d; viol=(v>HI)|(v<LO)
print('n',n,'status',r.status,'obj',r.fun,'d',d,'violations',viol.sum())
# iterative: drop violated & re-solve to estimate min violations
