# T-1941 W6: do Excel's (1.5,2] R-residual line tracks belong to the known K2/K3 chains? Count sweep peaks matched
# by each stage of efC (K2) and ss_fc (K3) evaluated on (1.5,2), at their own scale, vs. a shuffled-x control.
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lamscan import lines, alias
from lib6 import read_k2, read_k3
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>0.3]; xs=np.unique(d[:,0]); xi=np.searchsorted(xs,d[:,0])
P,Q=read_k2(); N3,D3=read_k3()
rng=np.random.default_rng(0)
for nm,c in (('K2.N',P[::-1]),('K2.D',Q[::-1]),('K3.N',N3),('K3.D',D3)):
    res=[]
    for st,f in lines(c,xs):
        hit=(np.abs(f[xi]-d[:,1])<0.0012).sum(); ctl=(np.abs(f[rng.permutation(len(xs))][xi]-d[:,1])<0.0012).sum()
        res.append('%s %d/%d'%(st,hit,ctl))
    print(nm,' '.join(res))
