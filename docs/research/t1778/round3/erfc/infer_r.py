# infer the stored double R with y = fl53(fl64(E*R)); save lo/hi candidate (consecutive doubles)
import numpy as np, sys
for name in ['runs','stepmap']:
    a=np.load(name+'.npy'); x,y,E,z,rE,rT,eE,erx,run=a.T
    r0=y/E
    f87=lambda E,r: (E.astype(np.longdouble)*r.astype(np.longdouble)).astype(np.float64)
    cands=[]
    for k in range(-4,5):
        r=r0.copy()
        for _ in range(abs(k)): r=np.nextafter(r,np.inf if k>0 else -np.inf)
        cands.append(np.where(f87(E,r)==y, r, np.nan))
    C=np.array(cands)
    cnt=np.sum(~np.isnan(C),0)
    lo=np.nanmin(C,0); hi=np.nanmax(C,0)
    print(name,'candidates per point:',np.bincount(cnt))
    np.save(name+'_R.npy',np.vstack([x,lo,hi,cnt,run,erx,y,E]).T)
