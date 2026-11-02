"""T-3571: FROZEN on w2's old (100,120) run (t2464-w2f-xruns-1, 1,024 cells; w4's best was 185 bit-exact), and the
fresh sweep's FROZEN misses beyond 15 digits with their term counts; exact rate by term count on the fresh sweep."""
import math, collections
import data, model
from runs import load
from score_final import w15
rows = data.load('t2464-w2f-xruns-1', 7)
ms = [model.cdf(r['x'], r['a'], r['b'], r['pdf']) for r in rows]
print('xruns-1 (100,120): FROZEN %d/%d/%d of %d' % (sum(m == r['cdf'] for m, r in zip(ms, rows)),
      sum(m == r['cdf'] or abs(m - r['cdf']) <= 4 * math.ulp(r['cdf']) for m, r in zip(ms, rows)),
      sum(w15(m, r['cdf']) for m, r in zip(ms, rows)), len(rows)))
by = collections.defaultdict(lambda: [0, 0])
for r in load('t3571-r5-bfrac-fresh'):
    y = 1 - r['x']; f, k = model.fraction(r['a'], r['b'], r['x'], y, r['a'] - (r['a'] + r['b']) * r['x'])
    m = (r['pdf'] * (r['x'] * y)) * f
    b = min(k // 10 * 10, 60); by[b][0] += m == r['cdf']; by[b][1] += 1
    if not w15(m, r['cdf']): print('  w15 miss', r['a'], r['b'], r['x'], 'terms', k, 'ulps', round((m - r['cdf']) / math.ulp(r['cdf'])))
print('fresh exact by terms (bins of 10):', {k: f'{v[0]}/{v[1]}' for k, v in sorted(by.items())})
