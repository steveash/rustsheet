import mpmath as mp, numpy as np
from sc import *
mp.mp.dps=40
N=candidate('N3');D=candidate('D3')
def f(x): x=mp.mpf(x); return mp.loggamma(x)/(x-1) if x!=1 else -mp.euler
def r(c,x): return mp.polyval([mp.mpf(v) for v in c[::-1]],mp.mpf(x))
for x in np.linspace(0.7,1.5,33):
    a=r(N,x)/r(D,x); print('%.3f %+.3e'%(x, float(a/f(x)-1)))
