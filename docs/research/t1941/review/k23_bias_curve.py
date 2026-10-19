# T-1941 review (Fable 5.1): full model-free bias curves (Excel GAMMALN.PRECISE vs lgamma, relative, in ulp-rel units) for K2 and K3, 100 bins; also in ulps of B.
import sys; sys.path.insert(0,'.')
from k23lib import *
import mpmath as mp; mp.mp.dps=30
which=sys.argv[1]; lo,hi=(2.0000000001,3.0000000001) if which=='K2' else (0.7,1.5)
x,B,C,cs=load_rows_B(lo,hi); nb=100; idx=np.minimum(((x-lo)/(hi-lo)*nb).astype(int),nb-1)
rng=np.random.default_rng(0)
print(which,': bin  n   mean_rel(ulp-rel) SE | mean in ulps of B | exponent of B at bin centre')
for b in range(nb):
    s=np.where(idx==b)[0]
    if len(s)<100: continue
    sub=rng.choice(s,min(2500,len(s)),replace=False); rel=[];ul=[]
    for i in sub:
        xi=mp.mpf(float(x[i])); tv=mp.loggamma(xi); bi=mp.mpf(float(B[i])); rel.append(float((bi-tv)/tv)/2**-53); ul.append(float((bi-tv)/mp.mpf(np.spacing(float(B[i])))))
    rel=np.array(rel); ul=np.array(ul); xc=lo+(hi-lo)*(b+0.5)/nb
    print('%.3f %6d %+.3f %.3f | %+.3f | %d'%(xc,len(s),rel.mean(),rel.std()/np.sqrt(len(rel)),ul.mean(),(np.floor(np.log2(abs(float(mp.loggamma(xc))))) if abs(xc-1)>1e-9 else 0)))
