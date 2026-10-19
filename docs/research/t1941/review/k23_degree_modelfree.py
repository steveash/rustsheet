# T-1941 review (Fable 5.1): model-free degree tests for K2 ((2,3], S = lgamma/(x-2), D0 = 2^-7) and K3 ([0.7,1.5), A = lgamma/(x-1), D0 = 2^-13).
# Target per bin: f_b = true(x_b)*(1+bias_b), bias_b = mean relative error of Excel GAMMALN.PRECISE vs lgamma in the bin.
import sys; sys.path.insert(0,'.')
from k23lib import *
import mpmath as mp; mp.mp.dps=60
which=sys.argv[1]
if which=='K2':
    lo,hi=2.0000000001,3.0000000001; D0=mp.mpf(2)**-7; degs=((7,7),(8,7),(8,8),(9,8),(8,9),(9,9)); sh=2
    tf=lambda t: mp.loggamma(t)/(t-2)
else:
    lo,hi=0.7,1.5; D0=mp.mpf(2)**-13; degs=((8,8),(9,8),(8,9),(9,9),(10,9)); sh=1
    tf=lambda t: mp.loggamma(t)/(t-1)
x,B,C,cs=load_rows_B(lo,hi)
nb=int(sys.argv[2]) if len(sys.argv)>2 else 80; idx=np.minimum(((x-lo)/(hi-lo)*nb).astype(int),nb-1)
rng=np.random.default_rng(0); xb=[];fb=[];se=[]
for b in range(nb):
    s=np.where(idx==b)[0]
    if len(s)<200: continue
    sub=rng.choice(s,min(3000,len(s)),replace=False)
    xc=mp.mpf(lo)+mp.mpf(hi-lo)*(2*b+1)/(2*nb)
    rel=[]
    for i in sub:
        xi=mp.mpf(float(x[i])); tv=mp.loggamma(xi); rel.append((mp.mpf(float(B[i]))-tv)/tv)
    m=sum(rel)/len(rel); sd=mp.sqrt(sum((r-m)**2 for r in rel)/len(rel))/mp.sqrt(len(rel))
    # relative bias of the output equals relative bias of S (the (x-c) factor is exact), up to the final rounding (zero-mean)
    xb.append(xc); fb.append(tf(xc)*(1+m)); se.append(sd)
print(which,'bins',len(xb),'bias ulp-rel samples:',['%.3f:%+.2f±%.2f'%(float(xx),float(f/tf(xx)-1)/2**-53,float(s)/2**-53) for xx,f,s in list(zip(xb,fb,se))[::10]])
def fit(n,m):
    rows=[];rhs=[]
    for xx,f,s in zip(xb,fb,se):
        w=1/(abs(f)*s); rows.append([w*xx**j for j in range(n+1)]+[-w*f*xx**j for j in range(1,m+1)]); rhs.append(w*f*D0)
    A=mp.matrix(rows); bvec=mp.matrix(rhs); sol=mp.lu_solve(A.T*A,A.T*bvec)
    N=[sol[j] for j in range(n+1)]; D=[D0]+[sol[n+1+j] for j in range(m)]
    res=np.array([float((sum(c*xx**j for j,c in enumerate(N))/sum(c*xx**j for j,c in enumerate(D))-f)/f/s) for xx,f,s in zip(xb,fb,se)])
    return res,N,D
for n,m in degs:
    res,N,D=fit(n,m)
    print('(%d,%d): chi2/dof %.2f max|res|/SE %.1f  top N %.3g top D %.3g'%(n,m,(res**2).sum()/max(1,len(res)-(n+m+1)),np.abs(res).max(),float(N[-1]),float(D[-1])))
