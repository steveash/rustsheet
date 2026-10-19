# T-1941 W6: locate the near-1 / (x-2) switch near 1.5.
import sys; sys.path.insert(0,sys.path[0])
from lib6 import *
X,B,C=load(sys.argv[1],float(sys.argv[2]),float(sys.argv[3]))
N,D=read_k3(); P,Q=read_k2()
r3,_=k3(X,N,D); r2,_=k2(X,P,Q); u=np.spacing(np.abs(B))
e3=(B-r3)/u; e2=(B-r2)/u
w=float(sys.argv[4])
for lo in np.arange(float(sys.argv[2]),float(sys.argv[3]),w):
    s=(X>=lo)&(X<lo+w)
    if s.sum()==0: continue
    print('%.6f %6d  K3 ex %.3f |e|<=2 %.3f med %+.1f | K2 ex %.3f |e|<=8 %.3f med %+.1f  %s'%(lo,s.sum(),(e3[s]==0).mean(),(abs(e3[s])<=2).mean(),np.median(e3[s]),(e2[s]==0).mean(),(abs(e2[s])<=8).mean(),np.median(e2[s]),','.join(sorted(set(C[s])))[:60]))
