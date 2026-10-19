# T-1941 W6: DE over log(-poles) of D (N by LS) maximising the soft line-match. Usage: polede.py PEAKS n d lo hi halfwidth sig maxiter seed [truthchain]
import sys, numpy as np, mpmath as mp
from scipy.optimize import differential_evolution
sys.path.insert(0,sys.path[0]); from poleobj import Obj; from rfit import fit; from synth import read_chain
mp.mp.dps=80
peaks,n,d,lo,hi,hw,sig,mi,seed=sys.argv[1],int(sys.argv[2]),int(sys.argv[3]),float(sys.argv[4]),float(sys.argv[5]),float(sys.argv[6]),float(sys.argv[7]),int(sys.argv[8]),int(sys.argv[9])
a,b,e=fit(n,d,mp.mpf(lo),mp.mpf(hi),iters=10)
p0=np.array(sorted([float(mp.re(r)) for r in mp.polyroots(b[::-1],maxsteps=200,extraprec=300)]))
o=Obj(peaks,n); L0=np.log(-p0)
if len(sys.argv)>10:
    N,D=read_chain(sys.argv[10]); pt=np.array(sorted([float(mp.re(r)) for r in mp.polyroots(D[::-1],maxsteps=200,extraprec=200)]))
    print('truth offset in log:',' '.join('%+.3f'%v for v in np.log(-pt)-L0),' truth hard %.3f'%o(pt,hard=True))
print('start hard %.3f'%o(p0,hard=True))
f=lambda z: -o(-np.exp(L0+z),sig=sig)
r=differential_evolution(f,[(-hw,hw)]*d,popsize=25,maxiter=mi,seed=seed,tol=1e-10,polish=False,updating='immediate',init='sobol')
pb=-np.exp(L0+r.x)
print('DE soft %.3f hard %.3f  offset %s'%(-r.fun,o(pb,hard=True),' '.join('%+.4f'%v for v in r.x)))
print('poles',' '.join('%.6g'%v for v in pb))
