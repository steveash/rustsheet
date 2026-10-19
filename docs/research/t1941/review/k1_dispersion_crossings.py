# T-1941 review (Fable 5.1): dispersion of (Excel - sim) per bin, and binade crossings of AN2's intermediates in [1,2).
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1]); y,g=load_rows('g_rows.txt')
m=G(N,D,y); e=(g-m)/np.spacing(np.abs(g))
nb=20; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
print('bin   miss%  mean   std   P(e=+1) P(e=-1) P(|e|>1)')
for b in range(nb):
    s=idx==b; v=e[s]
    print('%.2f %5.1f %+.3f %.3f  %.3f  %.3f  %.3f'%(1+b/nb,100*(v!=0).mean(),v.mean(),v.std(),(v==1).mean(),(v==-1).mean(),(np.abs(v)>1).mean()))
# binade crossings of stage values
yy=np.linspace(1,2,200001)
def stages(c,y):
    out=[]; s=np.full_like(y,c[-1])
    for ci in c[-2::-1]:
        p=st(L(s)*L(y)); s=st(L(p)+L(ci)); out.append(('p',p)); out.append(('s',s))
    return out
for nm,c in (('N',N),('D',D)):
    for k,(kind,v) in enumerate(stages(c,yy)):
        ex=np.floor(np.log2(np.abs(v))); ch=np.where(np.diff(ex)!=0)[0]
        if len(ch): print(nm,k//2,kind,'crossings at',['%.4f (2^%d->2^%d)'%(yy[i],ex[i],ex[i+1]) for i in ch])
