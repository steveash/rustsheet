# compare every R*.txt.mp (remez.py output) with fitAN2.txt after the best common scale of all coefs except D0
import numpy as np, math, glob, mpmath as mp
mp.mp.dps=50
a=[mp.mpf(float.fromhex(h)) for h in open('fitAN2.txt').read().split()]
idx=[i for i in range(18) if a[i]!=0 and i!=9]
for f in sorted(glob.glob('R*.txt.mp')):
    c=[mp.mpf(s) for s in open(f).read().split()]; full=c[:8]+[0,mp.mpf(-1)/16]+c[8:]+[0]
    # best common scale on all coefs except D0 (log-mean ratio)
    r=[a[i]/full[i] for i in idx]; s=sum(r)/len(r)
    du=[float((s*full[i]-a[i])/mp.mpf(math.ulp(float(a[i])))) for i in idx]
    print('%-28s scale-1 %.4e  D0 implied %.10g  resid ulps (max %.2e): %s'%(f,float(s-1),float(-s/16),max(abs(v) for v in du),' '.join('%.1e'%v for v in du)))
