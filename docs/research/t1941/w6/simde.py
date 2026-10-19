# T-1941 W6: DE over log(-poles) of D (N by LS) maximising the exact-simulation spectral objective (simobj.py).
# Usage: simde.py RFILE CACHE nseg n d lo hi halfwidth maxiter seed [truthchain]
import sys, numpy as np, mpmath as mp
from scipy.optimize import differential_evolution
sys.path.insert(0,sys.path[0]); from simobj import build, SimObj; from poleobj import dpoly, npoly; from rfit import fit; from synth import read_chain
mp.mp.dps=80
rf,cache,nseg,n,d,lo,hi,hw,mi,seed=sys.argv[1],sys.argv[2],int(sys.argv[3]),int(sys.argv[4]),int(sys.argv[5]),float(sys.argv[6]),float(sys.argv[7]),float(sys.argv[8]),int(sys.argv[9]),int(sys.argv[10])
o=SimObj(build(rf,cache,maxseg=nseg))
a,b,e=fit(n,d,mp.mpf(lo),mp.mpf(hi),iters=10)
p0=np.array(sorted([float(mp.re(r)) for r in mp.polyroots(b[::-1],maxsteps=200,extraprec=300)])); L0=np.log(-p0)
def val(z):
    D=dpoly(-np.exp(L0+z)); return o(npoly(D,n),D)
if len(sys.argv)>11:
    N,D=read_chain(sys.argv[11]); pt=np.array(sorted([float(mp.re(r)) for r in mp.polyroots(D[::-1],maxsteps=200,extraprec=200)]))
    print('truth offset',' '.join('%+.4f'%v for v in np.log(-pt)-L0),'truth(poles) %.3f'%val(np.log(-pt)-L0),flush=True)
print('start %.3f'%val(np.zeros(d)),flush=True)
best=[-1]
def cb(xk,convergence=0):
    v=val(xk)
    if v>best[0]+0.01: best[0]=v; print('  gen best %.3f %s'%(v,' '.join('%+.4f'%t for t in xk)),flush=True)
r=differential_evolution(lambda z:-val(z),[(-hw,hw)]*d,popsize=20,maxiter=mi,seed=seed,tol=1e-12,polish=False,updating='immediate',init='sobol',callback=cb)
print('DE %.3f offset %s'%(-r.fun,' '.join('%+.4f'%v for v in r.x)))
print('poles',' '.join('%.8g'%v for v in -np.exp(L0+r.x)))
