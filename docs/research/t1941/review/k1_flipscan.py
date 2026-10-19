# T-1941 review (Fable 5.1): single-coefficient ulp flips of AN2 scored on [1.80,1.97]: exact rate, std(e), P(|e|>1); plus P/S binade relation per stage there.
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1])
X=[];Gv=[]
for l in open('g_rows.txt'):
    p=l.split(); x=float.fromhex(p[0])
    if 1.80<=x<1.97: X.append(x); Gv.append(float.fromhex(p[1]))
y=np.array(X); g=np.array(Gv)
def sc(N,D):
    m=G(N,D,y); e=(g-m)/np.spacing(np.abs(g)); return (m==g).mean(),e.std(),(np.abs(e)>1).mean(),e.mean()
b=sc(N,D); print('base exact %.3f std %.3f P>1 %.3f mean %+.3f'%b)
res=[]
for nm,c in (('N',N),('D',D)):
    for j in range(len(c)):
        for d in (-4,-3,-2,-1,1,2,3,4):
            cc=list(c); cc[j]=cc[j]+d*np.spacing(cc[j])
            r=sc(cc,D) if nm=='N' else sc(N,cc)
            res.append((r[0],r[1],nm,j,d))
res.sort(reverse=True)
print('best 15 single flips by exact rate on [1.80,1.97):')
for r in res[:15]: print('  %s%d %+d: exact %.3f std %.3f'%(r[2],r[3],r[4],r[0],r[1]))
res.sort(key=lambda r:r[1])
print('best 10 by std:')
for r in res[:10]: print('  %s%d %+d: exact %.3f std %.3f'%(r[2],r[3],r[4],r[0],r[1]))
# binade relation of P and S per stage at y=1.85
for nm,c in (('N',N),('D',D)):
    yy=np.array([1.75,1.85,1.95]); s=np.full_like(yy,c[-1])
    for k,ci in enumerate(c[-2::-1]):
        p=st(L(s)*L(y[:0].tolist()+list(yy))) if False else st(L(s)*L(yy)); s=st(L(p)+L(ci))
        print(nm,'stage',k,'coef idx',len(c)-2-k,'exp(P)',np.floor(np.log2(np.abs(p))).astype(int),'exp(S)',np.floor(np.log2(np.abs(s))).astype(int),'exp(c)',int(np.floor(np.log2(abs(ci)))))
