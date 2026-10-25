import math, mpmath, collections
from lib import load_probe, D, R64, exp_v6, mpf, xl
from forms import ingredients
rows=[r for r in load_probe('probe1') if r['case']=='probe1C1']
ing=ingredients(rows)
c=collections.Counter()
for r,g in zip(rows,ing):
    L=mpf(D(g['ga']+D(g['gb']-g['gab'])))
    for kx,tx in (('txD',mpf(D(g['am1']*g['LNd']))),('txE',R64(g['am1']*g['LNd'])),('txEe',R64(g['am1']*g['LNe']))):
        for ke,E in (('Ed',mpf(D(tx-L))),('Ee',R64(tx-L))):
            hit=exp_v6(E)==r['excel']
            c[(kx,ke)]+=hit
            c[(kx,ke,'zone' if 0.018<r['x']<0.0315 else 'out')]+=hit
for k in sorted(c): print(k,c[k])
print(sum(1 for r in rows if 0.018<r['x']<0.0315))
