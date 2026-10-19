# T-1941 W6: search the function-null space of an (n,d) raw-x rational for R on [1.5,2] (D0 = 1) for the coefficient
# set whose stage lines explain Excel's sweep peaks. Objective: amplitude-weighted soft match of all peaks (Gaussian,
# width sigma) minus a penalty when the function leaves |rel err| <= 3e-16. Differential evolution (scipy).
import sys, numpy as np, mpmath as mp
from scipy.optimize import differential_evolution
sys.path.insert(0,sys.path[0]); from rfit import fit
mp.mp.dps=60
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>0.35]; xs=np.unique(d[:,0]); xi=np.searchsorted(xs,d[:,0]); f_obs=d[:,1]; amp=d[:,2]
n,dd=map(int,sys.argv[2].split(',')); sig=float(sys.argv[3]); seed=int(sys.argv[4]) if len(sys.argv)>4 else 0
a,b,e=fit(n,dd,mp.mpf(1.5),mp.mpf(2)); c0=np.array([float(v) for v in a]+[float(v) for v in b[1:]])
xf=np.linspace(1.5,2,60); Rt=np.array([float(mp.loggamma(mp.mpf(x))/((mp.mpf(x)-1)*(mp.mpf(x)-2))) if x!=2 else float(1-mp.euler) for x in xf])
def split(c): return c[:n+1],np.concatenate([[1.0],c[n+1:]])
def pv(c,x): return np.polyval(c[::-1],x)
def Rfun(c,x): N,D=split(c); return pv(N,x)/pv(D,x)
# relative Jacobian -> null directions
J=np.array([(Rfun(c0*(1+1e-7*(np.arange(len(c0))==i)),xf)-Rfun(c0,xf))/1e-7/Rt for i in range(len(c0))]).T
U,S,Vt=np.linalg.svd(J,full_matrices=False)
print('singular values',' '.join('%.1e'%s for s in S))
k=int(np.sum(S<1e-6*S[0])); V=Vt[-k:]; Sk=S[-k:]
print('null dims',k)
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def stage_lines(c,x):
    n_=len(c)-1; s=np.full_like(x,c[n_]); sd=np.zeros_like(x); L=[]
    for j in range(n_-1,-1,-1):
        P=s*x; Pd=sd*x+s; L.append(alias(Pd*2.0**-52/np.spacing(np.abs(P)))); s=P+c[j]; sd=Pd
        F=sd*2.0**-52/np.spacing(np.abs(s)); L.append(alias(F)); L.append(alias(F/2))
    return L
def coeffs(z):
    return c0*(1+V.T@z)
def obj(z,sig=sig):
    c=coeffs(z); N,D=split(c)
    err=np.abs(Rfun(c,xf)/Rt-1); pen=np.sum(np.maximum(err/3e-16-1,0)**2)
    best=np.zeros(len(f_obs))
    for Lf in stage_lines(N,xs)+stage_lines(D,xs):
        best=np.maximum(best,np.exp(-((Lf[xi]-f_obs)/sig)**2))
    return -(amp*best).sum()/amp.sum()+0.01*min(pen,1e6)
# bounds: function-preserving extent per direction (relative change <= 1e-16/S_k), capped at 2
bd=[(-min(2.0,3e-16/s),min(2.0,3e-16/s)) for s in Sk]
print('bounds',' '.join('%.1e'%b_[1] for b_ in bd))
print('base obj %.4f'%obj(np.zeros(k)))
r=differential_evolution(obj,bd,popsize=40,maxiter=int(sys.argv[5]) if len(sys.argv)>5 else 300,seed=seed,tol=1e-8,polish=False,workers=1,updating='immediate')
print('DE obj %.4f'%r.fun, 'z',' '.join('%.3e'%v for v in r.x))
c=coeffs(r.x); N,D=split(c)
print('hard match (tol .0012):',sum(1 for i in range(len(f_obs)) if any(abs(L[xi[i]]-f_obs[i])<0.0012 for L in stage_lines(N,xs)+stage_lines(D,xs))),'of',len(f_obs))
print('N',' '.join(v.hex() for v in N)); print('D',' '.join(v.hex() for v in D))
