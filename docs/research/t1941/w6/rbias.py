# T-1941 W6: per-run bias of Excel's inner R against the true R = lnG/((x-1)(x-2)), in ulps of R, and the spread.
import sys, numpy as np, mpmath as mp, collections
mp.mp.dps=40
rows=collections.defaultdict(list)
for l in open(sys.argv[1]):
    p=l.split(); rows[p[5]].append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3]),int(p[4])))
for cs,v in sorted(rows.items(),key=lambda kv:kv[1][0][0]):
    v=np.array(v); x=v[:,0]; Rm=(v[:,1]+v[:,2])/2; u=np.spacing(v[:,1])
    idx=np.linspace(0,len(x)-1,min(len(x),400)).astype(int)
    e=np.array([float((mp.mpf(Rm[i])-mp.loggamma(mp.mpf(x[i]))/((mp.mpf(x[i])-1)*(mp.mpf(x[i])-2)))/mp.mpf(u[i])) for i in idx])
    print('%-18s n=%6d x %.5f..%.5f  R-Rtrue mean %+.2f sd %.2f  (ncand1 %.2f)'%(cs,len(x),x.min(),x.max(),e.mean(),e.std(),(v[:,3]==1).mean()))
