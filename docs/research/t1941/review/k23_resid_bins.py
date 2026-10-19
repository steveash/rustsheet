# T-1941 review (Fable 5.1): bin-mean residuals (Excel - sim)/ulp for K3 (c5_f99t) on [0.7,1.5) and K2 (grid_a) on (2,3]; miss rate and std per bin.
import sys; sys.path.insert(0,'.')
from k23lib import *
x,B,C,cs=load_rows_B(0.7,1.5); r,A=k3(x); e=(B-r)/np.spacing(np.abs(B))
print('K3 c5_f99t: n=%d exact %.4f'%(len(x),(r==B).mean()))
nb=32; idx=np.minimum(((x-0.7)/0.8*nb).astype(int),nb-1)
print('bin    n    miss%  mean   SE    std  P>1')
for b in range(nb):
    s=idx==b; v=e[s]
    if s.sum(): print('%.3f %7d %5.1f %+.3f %.3f %.3f %.3f'%(0.7+b*0.8/nb,s.sum(),100*(v!=0).mean(),v.mean(),v.std()/np.sqrt(len(v)),v.std(),(np.abs(v)>1).mean()))
x,B,C,cs=load_rows_B(2.0,3.0000001); x=x[x>2]; 
x2,B2,_,_=load_rows_B(2.0,3.0000001); s=x2>2; x2=x2[s]; B2=B2[s]
y,S=k2(x2); e=(B2-y)/np.spacing(np.abs(B2))
print('K2 grid_a: n=%d exact %.4f'%(len(x2),(y==B2).mean()))
nb=20; idx=np.minimum(((x2-2)*nb).astype(int),nb-1)
print('bin    n    miss%  mean   SE    std  P>1')
for b in range(nb):
    s=idx==b; v=e[s]
    if s.sum(): print('%.3f %7d %5.1f %+.3f %.3f %.3f %.3f'%(2+b/nb,s.sum(),100*(v!=0).mean(),v.mean(),v.std()/np.sqrt(len(v)),v.std(),(np.abs(v)>1).mean()))
