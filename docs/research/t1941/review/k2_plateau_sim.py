# T-1941 review (Fable 5.1): plateau mechanism. (1) binade crossings of grid_a's K2 intermediates and of the output; (2) simulated chain bias
# (sim - exact rational) in ulps of output on a fine grid; (3) Excel's step positions (from bias_K2.txt) for comparison.
import sys; sys.path.insert(0,'.')
from k23lib import *
import mpmath as mp; mp.mp.dps=40
xx=np.linspace(2.0001,3.0,400001)
def stages(c_desc,x):
    out=[]; s=np.full_like(x,c_desc[0])
    for ci in c_desc[1:]:
        p=st(L(s)*L(x)); s=st(L(p)+L(ci)); out.append(p); out.append(s)
    return out
cr=[]
for nm,c in (('N',P2),('D',Q2)):
    for k,v in enumerate(stages(c,xx)):
        ex=np.floor(np.log2(np.abs(v))); ch=np.where(np.diff(ex)!=0)[0]
        for i in ch: cr.append((xx[i],'%s.%s%d'%(nm,'PS'[k%2],k//2+1),int(ex[i]),int(ex[i+1])))
y,S=k2(xx)
for arr,nm in ((S,'S'),(y,'out'),(st(L(xx)-L(2.0)),'m')):
    ex=np.floor(np.log2(np.abs(arr))); ch=np.where(np.diff(ex)!=0)[0]
    for i in ch: cr.append((xx[i],nm,int(ex[i]),int(ex[i+1])))
cr.sort(); print('grid_a binade crossings on (2,3]:'); print(' '.join('%.4f:%s'%(a,b) for a,b,_,_ in cr if a>2.01))
# simulated bias: sim output vs exact rational function of grid_a (mpmath), per 0.005 bin, mean in ulps of out
sub=xx[::40]; ys,Ss=k2(sub)
Pm=[mp.mpf(c) for c in P2]; Qm=[mp.mpf(c) for c in Q2]
def exact(x):
    x=mp.mpf(float(x)); n=mp.mpf(0); d=mp.mpf(0)
    for c in Pm: n=n*x+c
    for c in Qm: d=d*x+c
    return (x-2)*n/d
bias=np.array([float((mp.mpf(float(ys[i]))-exact(sub[i]))/mp.mpf(np.spacing(float(ys[i])))) for i in range(len(sub))])
nb=200; idx=np.minimum(((sub-2)*nb).astype(int),nb-1)
print('simulated grid_a chain bias (ulps of out), 200 bins:')
print(' '.join('%.3f:%+.2f'%(2+(b+0.5)/nb,bias[idx==b].mean()) for b in range(nb)))
