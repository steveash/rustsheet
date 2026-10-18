import sys, numpy as np
from scipy.optimize import milp, LinearConstraint, Bounds
from scipy.sparse import csr_matrix, hstack, identity, vstack
fn=sys.argv[1]; deg=int(sys.argv[2]); R=float(sys.argv[3]); tl=float(sys.argv[4]) if len(sys.argv)>4 else 60
Z=[];LO=[];HI=[]
for l in open(fn):
    a=l.split()
    if a[0]!='C': continue
    lo=float(a[2]); hi=float(a[3])
    if lo>-R or hi<R: Z.append(float.fromhex(a[1])); LO.append(lo); HI.append(hi)
Z=np.array(Z); LO=np.array(LO); HI=np.array(HI); n=len(Z)
A=np.vstack([(Z/0.25)**j for j in range(deg+1)]).T
M=4*R
# Ad - M b <= hi ; Ad + M b >= lo ; b binary
Ac=csr_matrix(A); I=identity(n,format='csr')*M
loA=np.where(LO>-R, LO, -np.inf); hiA=np.where(HI<R, HI, np.inf)
cons=[LinearConstraint(hstack([Ac,-I]), -np.inf, hiA-1e-3), LinearConstraint(hstack([Ac,I]), loA+1e-3, np.inf)]
# bound |G| <= R on a z grid
zg=np.linspace(0,0.25,41); Ag=np.vstack([(zg/0.25)**j for j in range(deg+1)]).T
cons.append(LinearConstraint(hstack([csr_matrix(Ag), csr_matrix((len(zg),n))]), -R, R))
c=np.concatenate([np.zeros(deg+1), np.ones(n)])
integ=np.concatenate([np.zeros(deg+1), np.ones(n)])
bnd=Bounds(np.concatenate([-np.inf*np.ones(deg+1), np.zeros(n)]), np.concatenate([np.inf*np.ones(deg+1), np.ones(n)]))
r=milp(c,constraints=cons,integrality=integ,bounds=bnd,options={'time_limit':tl,'disp':False})
d=r.x[:deg+1]; v=A@d; viol=(v>HI)|(v<LO)
print(fn,'n',n,'status',r.status,r.message[:40],'obj',r.fun,'d',np.round(d,3),'viol',viol.sum(), 'zviol', np.round(np.sort(Z[viol]),4)[:40])
