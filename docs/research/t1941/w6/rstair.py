# T-1941 W6: R staircase: mean (R_excel - Rtrue)/ulp(R) in bins over (1.5,2), all points (subsampled per bin).
import sys, numpy as np, mpmath as mp
mp.mp.dps=30
v=[]
for l in open(sys.argv[1]):
    p=l.split(); v.append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3])))
v=np.array(v); x=v[:,0]; Rm=(v[:,1]+v[:,2])/2; w=float(sys.argv[2]); rng=np.random.default_rng(1)
for lo in np.arange(1.5,2.0,w):
    s=np.where((x>=lo)&(x<lo+w))[0]
    if len(s)<20: continue
    s=rng.choice(s,min(len(s),int(sys.argv[3]) if len(sys.argv)>3 else 600),replace=False)
    e=np.array([float((mp.mpf(Rm[i])-mp.loggamma(mp.mpf(x[i]))/((mp.mpf(x[i])-1)*(mp.mpf(x[i])-2)))/mp.mpf(np.spacing(v[i,1]))) for i in s])
    print('%.4f n=%4d mean %+.2f se %.2f'%(lo,len(s),e.mean(),e.std()/np.sqrt(len(e))))
