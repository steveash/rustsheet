# T-1778: chain the per-sweep W phases (phases_iv.txt: name z lo hi) into the W polynomial around the erf-Taylor prior.
# Usage: python3 wfit.py ZMAX DEG
import math, sys
import numpy as np
from scipy.optimize import linprog
G=2.0**-53
K=1.1366973296084131
s=[2/math.sqrt(math.pi)*(-1)**k/(math.factorial(k)*(2*k+1)) for k in range(6)]
h=[0,0.14068131264230,0.0448191707,0.00180525,0.00018]
prior=[(h[k]-s[k])/K for k in range(1,5)]+[0,0]
print('prior c1..c4',prior)
zmax=float(sys.argv[1]); deg=int(sys.argv[2]); tol=0.006
obs=[]
for l in open('w_phases.txt'):
    nm,zs,a,b=l.split(); z=float(zs)
    if z<=zmax: obs.append((nm,z,float(a)-tol,float(b)+tol))
zr=1e-4
sc=[zr**(k+1) for k in range(deg)]
nom=np.array([prior[k]*sc[k]/G for k in range(deg)])
widths=[1e-10,1e-4,1e-2,0.1,1.0,5.0][:deg]
bounds=[(-widths[k]*sc[k]/G, widths[k]*sc[k]/G) for k in range(deg)]
def rows(z): return np.array([(z/zr)**(k+1) for k in range(deg)])
def lp(cons,c):
    A=[];b=[]
    for z,lo,hi in cons:
        r=rows(z); u0=r@nom; nr=abs(r[0])
        A.append(r/nr); b.append((hi-u0)/nr); A.append(-r/nr); b.append(-(lo-u0)/nr)
    return linprog(c,A_ub=np.array(A),b_ub=np.array(b),bounds=bounds,method='highs')
def urange(cons,z):
    r=rows(z); u0=r@nom; nr=abs(r[0])
    if not cons:
        lo=sum(min(bb[0]*r[k],bb[1]*r[k]) for k,bb in enumerate(bounds)); hi=sum(max(bb[0]*r[k],bb[1]*r[k]) for k,bb in enumerate(bounds))
        return u0+lo,u0+hi
    a=lp(cons,r/nr); b=lp(cons,-r/nr)
    if a.status or b.status: return None
    return a.fun*nr+u0,-b.fun*nr+u0
branches=[[]]
for nm,z,a,b in obs:
    nb=[]
    for cons in branches:
        rg=urange(cons,z)
        if rg is None: continue
        for k in range(math.floor(rg[0]-b)-1,math.ceil(rg[1]-a)+2):
            if k+b<rg[0] or k+a>rg[1]: continue
            c2=cons+[(z,k+a,k+b)]
            if urange(c2,z) is not None: nb.append(c2)
    branches=nb
    print(nm,'z %.3g'%z,'branches',len(branches)); sys.stdout.flush()
    if not branches or len(branches)>2000: break
for cons in branches[:4]:
    out=[]
    for k in range(deg):
        e=np.zeros(deg); e[k]=1
        a=lp(cons,e); b=lp(cons,-e)
        out.append('c%d [%.15g, %.15g]'%(k+1,(a.fun+nom[k])*G/sc[k],(-b.fun+nom[k])*G/sc[k]))
    print(' ; '.join(out))
