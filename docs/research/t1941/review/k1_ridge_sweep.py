# T-1941 review (Fable 5.1): ridge sweep of the two-scale correction: smallest integer moves that flatten the residual; exact score of each.
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1]); y,g=load_rows('g_rows.txt')
rng=np.random.default_rng(0); sub=rng.choice(len(y),400000,replace=False); ys=y[sub]; gs=g[sub]
nb=100; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
m=G(N,D,y); e=(g-m)/np.spacing(np.abs(g))
cnt=np.array([(idx==b).sum() for b in range(nb)]); ok=cnt>50
r=np.array([e[idx==b].mean() if cnt[b]>50 else 0 for b in range(nb)])[ok]; se=np.array([e[idx==b].std()/np.sqrt(cnt[b]) if cnt[b]>50 else 1 for b in range(nb)])[ok]
yb=(1+(np.arange(nb)+0.5)/nb)[ok]
Nv=np.polyval(N[::-1],yb); Dv=np.polyval(D[::-1],yb); uG=np.spacing(Nv/Dv)
W=[]; names=[]
for j in range(8): W.append(np.spacing(N[j])*yb**j/Dv/uG); names.append('N%d'%j)
for j in range(1,8): W.append(-np.spacing(D[j])*Nv*yb**j/Dv**2/uG); names.append('D%d'%j)
W=np.array(W).T; w=1/se; A=W*w[:,None]; b=r*w
def apply(di):
    N2=list(N); D2=list(D)
    for k,nm in enumerate(names):
        j=int(nm[1:])
        if nm[0]=='N': N2[j]=N2[j]+di[k]*np.spacing(N2[j])
        else: D2[j]=D2[j]+di[k]*np.spacing(D2[j])
    return N2,D2
print('lambda   |delta|max   fit-rms(ulp)  exact(sub)')
for lam in [1e4,3e3,1e3,3e2,1e2,30,10,3,1,0.3,0.1]:
    d=np.linalg.solve(A.T@A+lam*np.eye(15),A.T@b); di=np.rint(d)
    # greedy integer polish on the penalised objective
    def obj(t): return (((W@t-r)*w)**2).sum()+lam*(t**2).sum()
    best=obj(di); imp=True
    while imp:
        imp=False
        for j in range(15):
            for dd in (-1,1):
                t=di.copy(); t[j]+=dd; o=obj(t)
                if o<best: best=o; di=t; imp=True
    N2,D2=apply(di); m2=G(N2,D2,ys); fr=np.sqrt((((W@di-r))**2).mean())
    print('%7.1f  %8d   %.3f   %.4f  '%(lam,np.abs(di).max(),fr,(m2==gs).mean()),di.astype(int))
