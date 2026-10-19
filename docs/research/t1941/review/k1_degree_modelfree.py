# T-1941 review (Fable 5.1): model-free degree test for K1. Target f_b = Gamma(y_b)*(1+bias_b), bias from bin means of Excel GAMMA.
# Fit (n,n) raw-y rationals with D0 fixed by weighted LSQ in mpmath, report residual in ulps vs SE.
import sys; sys.path.insert(0,'.')
from k1lib import *
import mpmath as mp; mp.mp.dps=60
y,g=load_rows('g_rows.txt')
nb=int(sys.argv[1]) if len(sys.argv)>1 else 100; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
rng=np.random.default_rng(0)
yb=[];fb=[];se=[]
for b in range(nb):
    s=np.where(idx==b)[0]
    if len(s)<200: continue
    sub=rng.choice(s,min(4000,len(s)),replace=False)
    yc=mp.mpf(1)+mp.mpf(2*b+1)/(2*nb)
    # relative error of Excel vs Gamma at each point, averaged (model-free bias), then target at bin centre
    rel=[ (mp.mpf(float(g[i]))-mp.gamma(mp.mpf(float(y[i]))))/mp.gamma(mp.mpf(float(y[i]))) for i in sub]
    m=sum(rel)/len(rel); sd=mp.sqrt(sum((r-m)**2 for r in rel)/len(rel))/mp.sqrt(len(rel))
    yb.append(yc); fb.append(mp.gamma(yc)*(1+m)); se.append(sd)
print('bins',len(yb))
print('model-free bias (ulp-rel) at a few bins:',[(float(yy),'%+.3f±%.3f'%(float((f/mp.gamma(yy)-1)/mp.mpf(2)**-53),float(s/mp.mpf(2)**-53))) for yy,f,s in list(zip(yb,fb,se))[::12]])
def fit(n,m,D0):
    # unknowns: N0..Nn, D1..Dm ; rows: N(y) - f D(y) = f*D0  weighted by 1/(f*se)
    rows=[];rhs=[]
    for yy,f,s in zip(yb,fb,se):
        w=1/(f*s)
        rows.append([w*yy**j for j in range(n+1)]+[-w*f*yy**j for j in range(1,m+1)]); rhs.append(w*f*D0)
    A=mp.matrix(rows); bvec=mp.matrix(rhs)
    sol=mp.lu_solve(A.T*A, A.T*bvec)
    N=[sol[j] for j in range(n+1)]; D=[D0]+[sol[n+1+j] for j in range(m)]
    res=[]
    for yy,f,s in zip(yb,fb,se):
        Nv=sum(c*yy**j for j,c in enumerate(N)); Dv=sum(c*yy**j for j,c in enumerate(D))
        res.append(float((Nv/Dv-f)/f/s))   # in units of SE
    res=np.array(res); return res,N,D
for n,m in ((5,5),(6,6),(7,7),(8,8)):
    res,N,D=fit(n,m,mp.mpf(-1)/16)
    print('(%d,%d): chi2/dof %.2f  max|res|/SE %.1f'%(n,m,(res**2).sum()/(len(res)-(n+m+1)),np.abs(res).max()))
    if (n,m)==(7,7): print('   top coefficients N7 %.4g D7 %.4g; N0 %.6g D1 %.6g'%(float(N[7]),float(D[7]),float(N[0]),float(D[1])))
# compare the (7,7) LSQ function with AN2's function, and score the rounded LSQ set exactly
res,N,D=fit(7,7,mp.mpf(-1)/16)
NA,DA=load_cand('../g_candidate_w4_AN2.txt')
for yy in (mp.mpf('1.2'),mp.mpf('1.5'),mp.mpf('1.8')):
    fv=sum(c*yy**j for j,c in enumerate(N))/sum(c*yy**j for j,c in enumerate(D))
    av=sum(mp.mpf(c)*yy**j for j,c in enumerate(NA))/sum(mp.mpf(c)*yy**j for j,c in enumerate(DA))
    print('y=%.1f  fit/AN2-1 = %.3e   fit/Gamma-1 = %.3e (ulp %.3f)  AN2/Gamma-1 ulp %.3f'%(float(yy),float(fv/av-1),float(fv/mp.gamma(yy)-1),float((fv/mp.gamma(yy)-1)/mp.mpf(2)**-53),float((av/mp.gamma(yy)-1)/mp.mpf(2)**-53)))
print('fit N:',[float(c) for c in N]); print('fit D:',[float(c) for c in D])
Nf=[float(c) for c in N]; Df=[float(c) for c in D]
m=G(Nf,Df,y); print('rounded LSQ (7,7) exact score: %.4f within1 %.4f'%((m==g).mean(),(np.abs((g-m)/np.spacing(np.abs(g)))<=1).mean()))
open('lsq77.txt','w').write('N '+' '.join(float(c).hex() for c in Nf)+'\nD '+' '.join(float(c).hex() for c in Df)+'\n')
