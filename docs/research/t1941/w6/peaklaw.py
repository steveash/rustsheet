# T-1941 W6: fraction of per-segment peaks explained by the stage-law set of a chain (stagelaws: S, T, P, product
# sidebands P_n, sum sidebands S_m, quotient Q), vs a shuffled-x null. Usage: peaklaw.py CHAIN PEAKS tol amin
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from stagelaws import stage_F, quotient_F; from synth import read_chain
N,D=read_chain(sys.argv[1]); N=np.array(N); D=np.array(D)
d=np.loadtxt(sys.argv[2]); tol=float(sys.argv[3]); d=d[d[:,2]>float(sys.argv[4])]
xs=np.unique(d[:,0]); xi=np.searchsorted(xs,d[:,0])
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
fam={}
for ch,c in (('N',N),('D',D)):
    for nm,(F,v,vd) in stage_F(c,xs).items(): fam[ch+'.'+nm]=alias(F)
for nm,(F,v,vd) in quotient_F(N,D,xs).items(): fam['R.'+nm]=alias(F)
def explain(perm):
    hit=np.zeros(len(d),bool); who={}
    for nm,f in fam.items():
        h=np.abs(f[perm][xi]-d[:,1])<tol; hit|=h; who[nm]=h.sum()
    return hit,who
hit,who=explain(np.arange(len(xs)))
rng=np.random.default_rng(0); null=[explain(rng.permutation(len(xs)))[0].mean() for _ in range(5)]
print('peaks %d  explained %.3f  null %.3f+-%.3f'%(len(d),hit.mean(),np.mean(null),np.std(null)))
groups={}
for nm,c in who.items():
    g=nm.split('.')[0]+'.'+''.join(ch for ch in nm.split('.')[1] if not ch.isdigit() and ch not in '+-')
    groups[g]=groups.get(g,0)+c
print(sorted(groups.items(),key=lambda kv:-kv[1]))
print('top laws',sorted(who.items(),key=lambda kv:-kv[1])[:15])
for amin in (0.5,0.7):
    m=d[:,2]>amin; print('amp>%.1f: n=%d explained %.3f'%(amin,m.sum(),hit[m].mean()))
