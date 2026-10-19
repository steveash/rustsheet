# Estimate Excel's function deviation Delta_E(x) = Excel_A/lgamma_A - 1 (relative in A):
# Delta_c5(x) (exact c5 function vs truth) + smoothed per-bin mean of (B - sim_c5)/(t*A).
import numpy as np, mpmath as mp
from sc import load, res, candidate
mp.mp.dps=30
x,b,c=load(); N=candidate('N3'); D=candidate('D3')
s=np.abs(x-1)>0.01
x=x[s]; b=b[s]
r=res(x,N,D); A=r/(x-1)
d=(b-r)/(x-1)/A          # relative deviation of Excel from c5 simulated, in A
edges=np.arange(0.7,1.5001,0.02); out=[]
Nm=[mp.mpf(v) for v in N[::-1]]; Dm=[mp.mpf(v) for v in D[::-1]]
for lo,hi in zip(edges[:-1],edges[1:]):
    m=(x>=lo)&(x<hi)
    if m.sum()<200: continue
    xc=(lo+hi)/2; xm=mp.mpf(xc)
    dc5=mp.polyval(Nm,xm)/mp.polyval(Dm,xm)/(mp.loggamma(xm)/(xm-1))-1
    dd=d[m].mean(); se=d[m].std()/np.sqrt(m.sum())
    # runs in bin
    out.append((xc,float(dc5),dd,float(dc5)+dd,m.sum()))
    print('%.3f  c5-true %+.2e  excel-c5 %+.2e  => Excel-true %+.2e  (n=%d)'%out[-1])
np.save('deltaE.npy',np.array(out))
