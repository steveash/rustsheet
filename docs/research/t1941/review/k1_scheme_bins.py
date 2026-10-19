# T-1941 review (Fable 5.1): score AN2 under three rounding schemes; bin-mean residual (Excel - sim)/ulp in 100 bins with SE.
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1]); y,g=load_rows('g_rows.txt')
for sch in ['stored','psum','reg']:
    m=G(N,D,y,sch); e=(g-m)/np.spacing(np.abs(g))
    print(sch,'exact %.4f within1 %.4f'%((m==g).mean(),(np.abs(e)<=1).mean()))
m=G(N,D,y,'stored'); e=(g-m)/np.spacing(np.abs(g))
nb=50; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
print('bin  n   mean(Excel-sim) ulp  SE   | mean(Excel-Gamma)?: see qc1b')
for b in range(nb):
    s=idx==b; v=e[s]
    print('%.2f %7d %+.4f %.4f'%(1+b/nb,s.sum(),v.mean(),v.std()/np.sqrt(len(v))))
