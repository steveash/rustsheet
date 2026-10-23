"""Coverage of a constant exponent shift c per (case, a, x) over b: how many cells' intervals contain c.
A tx error would show as a peak away from c = 0."""
import collections, sys
import numpy as np
sys.argv=[sys.argv[0]]+sys.argv[1:]
exec(open('interval1.py').read().split("groups = collections.defaultdict(list)")[0])
g2=collections.defaultdict(list)
for i in idx:
    tx, ty, lnB = E_parts(i)
    E = R64(tx + R64(ty - lnB))
    iv = interval(i, E)
    if iv: g2[(rows[i]['case'][-12:], A[i], X[i])].append(iv)
cs=np.arange(-3,3.001,0.05)
for k,v in sorted(g2.items()):
    cov=[sum(lo<=c<=hi for lo,hi in v) for c in cs]
    j=int(np.argmax(cov))
    print(k, len(v), 'cov@0', cov[60], 'best c', round(cs[j],2), cov[j])
