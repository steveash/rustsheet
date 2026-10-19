# T-1941 review (Fable 5.1): (1.5,2] GAMMALN glue. Is fl64(B + C) (GAMMALN(x)+LN(x)) a 53-bit double? Compare B+C with lgamma(x+1).
import sys; sys.path.insert(0,'.')
from k23lib import *
import mpmath as mp; mp.mp.dps=30
x,B,C,cs=load_rows_B(1.5000000001,2.0000000001)
ok=~np.isnan(C); x=x[ok]; B=B[ok]; C=C[ok]; cs=cs[ok]
print('rows in (1.5,2] with LN column:',len(x),'cases:',sorted(set(cs))[:12],'...')
s64=L(B)+L(C); rep=(s64.astype(np.float64).astype(L)==s64)
print('fraction where B+C is exactly representable in 53 bits: %.4f'%rep.mean())
# compare with lgamma(fl(x+1)) and lgamma(x+1) on a subsample
rng=np.random.default_rng(0); sub=rng.choice(len(x),min(20000,len(x)),replace=False)
d1=[];d2=[]
for i in sub:
    xp=float(x[i]); v=float(s64[i])
    t1=mp.loggamma(mp.mpf(xp)+1); t2=mp.loggamma(mp.mpf(float(st(L(xp)+L(1.0)))))
    d1.append(float((mp.mpf(v)-t1)/mp.mpf(np.spacing(v)))); d2.append(float((mp.mpf(v)-t2)/mp.mpf(np.spacing(v))))
d1=np.array(d1); d2=np.array(d2)
print('(B+C) - lgamma(x+1) in ulps of (B+C): mean %+.3f std %.3f; vs lgamma(fl(x+1)): mean %+.3f std %.3f'%(d1.mean(),d1.std(),d2.mean(),d2.std()))
# alternative: res = fl(Gb - L) with Gb a double: then B + C exact. Else maybe B = fl(something else). Check also B vs lgamma(x) directly
d0=[]
for i in sub:
    xp=float(x[i]); d0.append(float((mp.mpf(float(B[i]))-mp.loggamma(mp.mpf(xp)))/mp.mpf(np.spacing(float(B[i])))))
d0=np.array(d0); print('B - lgamma(x) in ulps of B: mean %+.3f std %.3f, |d|<=0.5: %.3f'%(d0.mean(),d0.std(),(np.abs(d0)<=0.5).mean()))
