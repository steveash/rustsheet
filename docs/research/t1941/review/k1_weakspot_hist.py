# T-1941 review (Fable 5.1): per-case error histogram of AN2 in [1.78,1.97]; within-run structure of the error (lag autocorrelation).
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1])
X=[];Gv=[];C=[]
for l in open('g_rows.txt'):
    p=l.split(); x=float.fromhex(p[0])
    if 1.78<=x<1.97: X.append(x); Gv.append(float.fromhex(p[1])); C.append(p[2])
y=np.array(X); g=np.array(Gv); C=np.array(C)
m=G(N,D,y); e=np.rint((g-m)/np.spacing(np.abs(g))).astype(int)
for c in sorted(set(C)):
    s=C==c; v=e[s]; ys=y[s]
    h={k:int((v==k).sum()) for k in range(-4,5)}
    # order by y and autocorrelation at lag 1 of the error
    o=np.argsort(ys); vv=v[o]-v[o].mean(); ac=(vv[:-1]*vv[1:]).mean()/max(vv.var(),1e-9)
    print('%s y=[%.4f,%.4f] n=%5d exact=%.2f  hist(-4..4)=%s  ac1=%.2f'%(c,ys.min(),ys.max(),s.sum(),(v==0).mean(),[h[k] for k in range(-4,5)],ac))
