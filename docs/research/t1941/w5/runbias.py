# per-run mean of (Excel B - true lgamma) and (Excel B - c5 exact function*(x-1)) in ulps of B
import mpmath as mp, numpy as np, collections
from sc import *
mp.mp.dps=30
x,b,c=load(); N=candidate('N3');D=candidate('D3')
br=np.where(np.nextafter(x[:-1],np.inf)!=x[1:])[0]+1
segs=np.split(np.arange(len(x)),br)
Nm=[mp.mpf(v) for v in N[::-1]];Dm=[mp.mpf(v) for v in D[::-1]]
out=[]
for s in segs:
    if len(s)<500: continue
    idx=s[np.linspace(0,len(s)-1,64).astype(int)]
    e1=[];e2=[]
    for i in idx:
        xx=mp.mpf(x[i]);
        if xx==1: continue
        tv=mp.loggamma(xx); mv=(xx-1)*mp.polyval(Nm,xx)/mp.polyval(Dm,xx); u=np.spacing(abs(b[i]))
        e1.append(float((b[i]-tv)/u)); e2.append(float((b[i]-mv)/u))
    out.append((x[s[0]],len(s),np.mean(e1),np.mean(e2),float(mp.loggamma(x[s[0]])/(x[s[0]]-1)) if x[s[0]]!=1 else 0))
    print('%.6f %6d true %+.3f c5 %+.3f'%(out[-1][:4]))
