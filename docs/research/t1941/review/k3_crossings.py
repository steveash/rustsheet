# T-1941 review (Fable 5.1): c5's K3 intermediate binade crossings on [0.7,1.5) (to compare with Excel's K3 staircase steps).
import sys; sys.path.insert(0,'.')
from k23lib import *
xx=np.linspace(0.7,1.5,320001)
def stages(c_asc,x):
    out=[]; s=np.full_like(x,c_asc[-1])
    for ci in c_asc[-2::-1]:
        p=st(L(s)*L(x)); s=st(L(p)+L(ci)); out.append(p); out.append(s)
    return out
cr=[]
for nm,c in (('N',N3),('D',D3)):
    for k,v in enumerate(stages(c,xx)):
        ex=np.floor(np.log2(np.abs(v)+1e-300)); ch=np.where(np.diff(ex)!=0)[0]
        for i in ch: cr.append((xx[i],'%s.%s%d'%(nm,'PS'[k%2],k//2+1)))
r,A=k3(xx)
for arr,nm in ((A,'A'),(r,'res')):
    ex=np.floor(np.log2(np.abs(arr)+1e-300)); ch=np.where(np.diff(ex)!=0)[0]
    for i in ch:
        if abs(xx[i]-1)>0.002: cr.append((xx[i],nm))
cr.sort(); print(' '.join('%.4f:%s'%(a,b) for a,b in cr))
