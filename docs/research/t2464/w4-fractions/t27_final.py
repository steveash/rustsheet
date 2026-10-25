"""Final scores of TOMS-X L140 (modelled pieces: w1 M1 density for bup's brcmp1, TOMS bpser with an Excel-style
prefactor) on every w2 direct run block on the L140 path, by block and even/odd, plus CR and the oracle-piece variant."""
import json, collections
from common4 import *
import tomsx
out = json.load(open('probe/w4p1.out.json'))['sheets']; meta = json.load(open('probe/w4p1.meta.json'))
pieces = {(m[2], m[3], m[4]): (out[m[0]][f'B{m[1]}']['v'], out[m[0]][f'C{m[1]}']['v']) for m in meta}
rows = load(lambda d: d['src'] == 'w2' and d['branch'] == 'direct' and 'xruns' in d['blk'] and d['a'] > 1 and d['b'] > 1)
rows = [r for r in rows if tomsx.toms_path(r['x'], r['a'], r['b']) == 'L140']
tomsx.prefetch('LN', [r['x'] for r in rows] + [1 - r['x'] for r in rows])
g = collections.defaultdict(list)
for r in rows: g[(r['blk'], r['a'], r['b'])].append(r)
T = collections.Counter()
for k, rs in sorted(g.items()):
    es = [r['cdf'] for r in rs]
    m = [tomsx.l140_best(r['x'], r['a'], r['b']) for r in rs]
    mo = [tomsx.l140_best(r['x'], r['a'], r['b'], *pieces[(r['x'], r['a'], r['b'])]) if (r['x'], r['a'], r['b']) in pieces else float('nan') for r in rs]
    s = score(m, es); so = score(mo, es); sc = score([r['cdfcr'] for r in rs], es)
    ev = sum(a == b for i, (a, b) in enumerate(zip(m, es)) if i % 2 == 0); od = s[1] - ev
    print(k, 'n %d | model exact %d (even %d odd %d) w4 %d w15 %d | oracle pieces exact %d w4 %d | CR exact %d w4 %d w15 %d' % (s[0], s[1], ev, od, s[2], s[3], so[1], so[2], sc[1], sc[2], sc[3]))
    for j, v in enumerate(s): T['m%d' % j] += v
    for j, v in enumerate(so): T['o%d' % j] += v
    for j, v in enumerate(sc): T['c%d' % j] += v
print(dict(T))
