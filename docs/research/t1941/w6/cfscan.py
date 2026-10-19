# T-1941 W6: common-factor null-manifold scan. Excel's chain = (N0*C)/(D0*C), N0/D0 a minimal fit, C = prod(x - r_i),
# normalised so that D's constant term is 1 (power-of-two habit of K2/K3). Score: sweep peaks matched by any stage line.
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from rfit import fit; from lamscan import lines
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>0.35]; xs=np.unique(d[:,0]); xi=np.searchsorted(xs,d[:,0]); tol=0.0012
def score(N,D):
    hit=np.zeros(len(d),bool)
    for c in (N,D):
        for nm,f in lines(c,xs): hit|=np.abs(f[xi]-d[:,1])<tol
    return hit.sum()
def mul(a,r):  # a ascending times (x - r)
    out=[0.0]*(len(a)+1)
    for i,v in enumerate(a): out[i+1]+=v; out[i]-=r*v
    return out
n,dd=map(int,sys.argv[2].split(',')); a,b,e=fit(n,dd,mp.mpf(1.5),mp.mpf(2)); a=[float(v) for v in a]; b=[float(v) for v in b]
print('base (%d,%d) err %.1e score %d of %d peaks'%(n,dd,float(e),score(a,b),len(d)))
rs=np.concatenate([-np.logspace(-3,4,3000),np.logspace(-3,4,3000)])
res=[]
for r in rs:
    N=mul(a,r); D=mul(b,r); s=1.0/D[0]; N=[v*s for v in N]; D=[v*s for v in D]
    res.append((score(N,D),r))
res.sort(reverse=True); sc=[r[0] for r in res]
print('k=1: best',res[:8],' median %d p99 %d'%(np.median(sc),np.percentile(sc,99)))
def detail(N,D):
    out=[]
    for ch,c in (('N',N),('D',D)):
        for nm,f in lines(c,xs):
            h=(np.abs(f[xi]-d[:,1])<tol).sum()
            if h>15: out.append('%s%s:%d'%(ch,nm,h))
    return ' '.join(out)
for s,r in res[:3]:
    N=mul(a,r); D=mul(b,r); k=1.0/D[0]; print(r,detail([v*k for v in N],[v*k for v in D]))
