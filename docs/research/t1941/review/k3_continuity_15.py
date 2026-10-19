# T-1941 review (Fable 5.1): does the near-1 kernel continue past 1.5? Fine bins of (Excel - c5 sim)/ulp and of model-free (Excel - lgamma)/ulp across [1.40,1.70].
import sys; sys.path.insert(0,'.')
from k23lib import *
import mpmath as mp; mp.mp.dps=30
x,B,C,cs=load_rows_B(1.40,1.70)
r,A=k3(x); e=(B-r)/np.spacing(np.abs(B))
rng=np.random.default_rng(0)
nb=30; idx=np.minimum(((x-1.4)/0.3*nb).astype(int),nb-1)
print('bin     n   c5-exact  mean(Excel-c5)  std   | model-free mean(Excel-lgamma) ulp  SE (subsample)')
for b in range(nb):
    s=np.where(idx==b)[0]
    if len(s)<50: continue
    v=e[s]; sub=rng.choice(s,min(3000,len(s)),replace=False)
    d=np.array([float((mp.mpf(float(B[i]))-mp.loggamma(mp.mpf(float(x[i]))))/mp.mpf(np.spacing(float(B[i])))) for i in sub])
    print('%.3f %6d  %.3f   %+.3f  %.3f  |  %+.3f  %.3f'%(1.4+b*0.3/nb,len(s),(r[s]==B[s]).mean(),v.mean(),v.std(),d.mean(),d.std()/np.sqrt(len(d))))
