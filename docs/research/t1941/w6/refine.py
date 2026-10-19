# T-1941 W6: local refinement of a chain in coefficient space with the exact-simulation spectral objective plus a
# function penalty (|N/D / Rtrue - 1| <= tol on 24 nodes, mpmath). Params: relative offsets of all coefficients except
# D0 (= 1). Optimiser: Powell from scipy, then report. Usage: refine.py RFILE CACHE nseg chainfile out tol maxfev
import sys, numpy as np, mpmath as mp
from scipy.optimize import minimize
sys.path.insert(0,sys.path[0]); from simobj import build, SimObj; from synth import read_chain
mp.mp.dps=40
o=SimObj(build(sys.argv[1],sys.argv[2],maxseg=int(sys.argv[3])))
N0,D0=read_chain(sys.argv[4]); N0=np.array(N0); D0=np.array(D0); s=1/D0[0]; N0*=s; D0*=s
tol=float(sys.argv[6])
XF=[mp.mpf(1.5)+mp.mpf(k+0.5)/48 for k in range(24)]; RT=[mp.loggamma(x)/((x-1)*(x-2)) for x in XF]
nN=len(N0)
def unpack(z): return N0*(1+z[:nN]), np.concatenate([[1.0],D0[1:]*(1+z[nN:])])
def ferr(N,D):
    return max(abs(mp.polyval([mp.mpf(v) for v in N[::-1]],x)/mp.polyval([mp.mpf(v) for v in D[::-1]],x)/r-1) for x,r in zip(XF,RT))
def f(z):
    N,D=unpack(z); e=float(ferr(N,D)); pen=max(0.0,e/tol-1)**2
    return -o(N,D)+pen
z0=np.zeros(nN+len(D0)-1)
print('start obj %.3f ferr %.1e'%(o(N0,D0),float(ferr(N0,D0))),flush=True)
r=minimize(f,z0,method='Powell',options={'maxfev':int(sys.argv[7]),'xtol':1e-9,'ftol':1e-4})
N,D=unpack(r.x)
print('end obj %.3f ferr %.1e nfev %d'%(o(N,D),float(ferr(N,D)),r.nfev))
open(sys.argv[5],'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')
