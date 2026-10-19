# T-1941 review (Fable 5.1): per-bin miss rate vs |bin-mean residual|; per-stage blame of AN2 misses (which single-stage +-1 ulp flip fixes a miss).
import sys; sys.path.insert(0,'.')
from k1lib import *
N,D=load_cand(sys.argv[1]); y,g=load_rows('g_rows.txt')
m=G(N,D,y); e=(g-m)/np.spacing(np.abs(g)); miss=m!=g
nb=20; idx=np.minimum(((y-1)*nb).astype(int),nb-1)
print('bin   n    miss%  mean_res  |mean|+? ')
for b in range(nb):
    s=idx==b; print('%.2f %7d %5.1f %+.3f'%(1+b/nb,s.sum(),100*miss[s].mean(),e[s].mean()))
# stage blame on a subsample of misses
rng=np.random.default_rng(1); sel=np.where(miss)[0]; sel=rng.choice(sel,min(40000,len(sel)),replace=False)
ys=y[sel]; gs=g[sel]; fixed=np.zeros(len(sel),bool); blame={}
for chain,c in (('N',N),('D',D)):
    for k in range(len(c)-1):
        for kind in 'ps':
            for dlt in (-1,1):
                fl=(k,kind,dlt)
                mm=G(N,D,ys,'stored',flipN=fl if chain=='N' else None,flipD=fl if chain=='D' else None)
                ok=mm==gs; blame[(chain,k,kind,dlt)]=ok; 
for dlt in (-1,1):
    mm=G(N,D,ys,divflip=dlt); blame[('div',0,'q',dlt)]=mm==gs
any_fix=np.zeros(len(sel),bool)
for k,v in blame.items(): any_fix|=v
print('misses sampled',len(sel),'fixed by some single stage flip: %.3f'%any_fix.mean())
idxs=np.minimum(((ys-1)*10).astype(int),9)
print('per tenth: fraction of misses fixed by any single flip, and top stages')
for b in range(10):
    s=idxs==b
    if s.sum()==0: continue
    tops=sorted(((v[s].mean(),k) for k,v in blame.items()),reverse=True)[:4]
    print('%.1f n=%5d anyfix=%.2f '%(1+b/10,s.sum(),any_fix[s].mean()),' '.join('%s%d%s%+d:%.2f'%(k[0],k[1],k[2],k[3],f) for f,k in tops))
