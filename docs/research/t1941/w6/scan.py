# T-1941 W6: exact rate of K3 (ss_fc), K2 (efC, grid_a) per bin over (1.3,2.2).
import sys; sys.path.insert(0,sys.path[0])
from lib6 import *
X,B,C=load(sys.argv[1],1.3,2.2)
N,D=read_k3(); P,Q=read_k2(); Pg,Qg=read_k2(os.path.join(HERE,'..','w2','grid_a.txt')) if os.path.exists(os.path.join(HERE,'..','w2','grid_a.txt')) else (None,None)
r3,_=k3(X,N,D); r2,_=k2(X,P,Q)
u=np.spacing(np.abs(B))
for lo in np.arange(1.30,2.2,0.025):
    s=(X>=lo)&(X<lo+0.025)
    if s.sum()==0: continue
    e3=(B[s]-r3[s])/u[s]; e2=(B[s]-r2[s])/u[s]
    print('%.3f %6d  K3 ex %.3f mean %+.2f rms %.2f | K2 ex %.3f mean %+.2f rms %.2f'%(lo,s.sum(),(e3==0).mean(),e3.mean(),np.sqrt((e3**2).mean()),(e2==0).mean(),e2.mean(),np.sqrt((e2**2).mean())))
