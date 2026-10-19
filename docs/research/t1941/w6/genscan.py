# T-1941 W6: generator scan. For each fit interval [lo,hi] and degree (n,d), a relative-LS rational of R; score the
# hard line-match of its chain (D0 = 1; scale-free frequencies) on a peak file. Usage: genscan.py PEAKS n,d LOS HIS
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from poleobj import Obj, stage_lines; from rfit import fit
mp.mp.dps=80
o=Obj(sys.argv[1],9)
def hard(N,D):
    best=np.zeros(len(o.f))
    for Lf in stage_lines(N,o.xs)+stage_lines(D,o.xs): best=np.maximum(best,(np.abs(Lf[o.xi]-o.f)<0.0012))
    return (o.a*best).sum()/o.a.sum()
n,d=map(int,sys.argv[2].split(','))
res=[]
for lo in [float(v) for v in sys.argv[3].split(',')]:
    for hi in [float(v) for v in sys.argv[4].split(',')]:
        try: a,b,e=fit(n,d,mp.mpf(lo),mp.mpf(hi),iters=8)
        except Exception: continue
        N=np.array([float(v/b[0]) for v in a]); D=np.array([float(v/b[0]) for v in b])
        s=hard(N,D); res.append((s,lo,hi)); print('%.2f %.2f %.3f'%(lo,hi,s),flush=True)
res.sort(reverse=True); print('BEST',res[:5])
