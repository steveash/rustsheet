# T-1941 review (Fable 5.1): two-scale correction of AN2. Measure r(y)=mean(Excel-sim) per bin, solve integer ulp moves of the 15 free
# coefficients (D0 fixed) so that the linearised function change matches r, then re-score exactly.
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1]); y,g=load_rows('g_rows.txt')
nb=100; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
def resid(N,D):
    m=G(N,D,y); e=(g-m)/np.spacing(np.abs(g))
    r=np.array([e[idx==b].mean() if (idx==b).sum()>50 else np.nan for b in range(nb)]); se=np.array([e[idx==b].std()/np.sqrt((idx==b).sum()) if (idx==b).sum()>50 else np.nan for b in range(nb)])
    return r,se,(m==g).mean()
r,se,ex=resid(N,D); print('AN2 exact %.4f; residual rms %.3f ulp, max %.3f'%(ex,np.sqrt((r**2).mean()),np.abs(r).max()))
ok=~np.isnan(r); r=r[ok]; se=se[ok]; yb=(1+(np.arange(nb)+0.5)/nb)[ok]; print('bins used',ok.sum())
# sensitivities in ulps of G per ulp of coefficient (exact function, float64 fine)
Nv=np.polyval(N[::-1],yb); Dv=np.polyval(D[::-1],yb); Gv=Nv/Dv; uG=np.spacing(Gv)
W=[]; names=[]
for j in range(8): W.append(np.spacing(N[j])*yb**j/Dv/uG); names.append('N%d'%j)
for j in range(1,8): W.append(-np.spacing(D[j])*Nv*yb**j/Dv**2/uG); names.append('D%d'%j)
W=np.array(W).T   # nb x 15
w=1/se; A=W*w[:,None]; b=r*w
U,s,Vt=np.linalg.svd(A,full_matrices=False); print('singular values',np.array2string(s,precision=2))
for keep in (6,8,10,12,15):
    si=np.where(np.arange(len(s))<keep,1/s,0); d=Vt.T@(si*(U.T@b))
    print('keep %2d: real delta (ulps):'%keep,np.array2string(d,precision=1,max_line_width=200),' fit rms %.3f'%np.sqrt((((W@d-r)*w)**2).mean()/(w**2).mean()))
# integer rounding + greedy refinement on the weighted LSQ objective, for keep=10
si=np.where(np.arange(len(s))<int(sys.argv[2]) if len(sys.argv)>2 else 10,1/s,0); d=Vt.T@(si*(U.T@b)); di=np.rint(d)
def obj(di): return (((W@di-r)*w)**2).sum()
best=obj(di); improved=True
while improved:
    improved=False
    for j in range(15):
        for dd in (-2,-1,1,2):
            t=di.copy(); t[j]+=dd; o=obj(t)
            if o<best-1e-9: best=o; di=t; improved=True
print('integer delta:',di.astype(int),'obj %.1f (chi2/n=%.2f)'%(best,best/len(r)))
N2=list(N); D2=list(D)
for k,nm in enumerate(names):
    j=int(nm[1:]); 
    if nm[0]=='N': N2[j]=N2[j]+di[k]*np.spacing(N2[j])
    else: D2[j]=D2[j]+di[k]*np.spacing(D2[j])
r2,se2,ex2=resid(N2,D2); r2=r2[ok]; print('corrected exact %.4f; residual rms %.3f ulp, max %.3f'%(ex2,np.sqrt((r2**2).mean()),np.abs(r2).max()))
m=G(N2,D2,y)
for k in range(10):
    s=(y>=1+k/10)&(y<1+(k+1)/10); print(' %.1f: %.3f'%(1+k/10,(m[s]==g[s]).mean()), end='')
print()
open('an2_corr.txt','w').write('N '+' '.join(float(v).hex() for v in N2)+'\nD '+' '.join(float(v).hex() for v in D2)+'\n')
