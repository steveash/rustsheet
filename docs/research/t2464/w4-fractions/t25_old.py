"""Hold-out: TOMS-X L140 (modelled pieces) on beta_exact's direct cdf cells (never used for tuning), and the
TOMS 708 branch map of all direct cdf cells with a, b > 1."""
import collections, sys
from common4 import *
import tomsx
rows = load(lambda d: d['src'] == 'old' and d['branch'] == 'direct' and not math.isnan(d['cdf']) and d['a'] > 1 and d['b'] > 1)
paths = collections.Counter(tomsx.toms_path(r['x'], r['a'], r['b']) for r in rows)
print('old direct a,b>1 by TOMS path', dict(paths))
sel = [r for r in rows if tomsx.toms_path(r['x'], r['a'], r['b']) == 'L140']
tomsx.prefetch('LN', [r['x'] for r in sel] + [1 - r['x'] for r in sel])
ms = []
for r in sel:
    try: ms.append(tomsx.l140_best(r['x'], r['a'], r['b']))
    except Exception as e: ms.append(float('nan'))
es = [r['cdf'] for r in sel]
print('L140 cells n %d: TOMS-X exact %d w4 %d w15 %d' % score(ms, es), '| CR exact %d w4 %d w15 %d' % score([r['cdfcr'] for r in sel], es)[1:])
for r, m in zip(sel, ms):
    print(f"  x={r['x']} a={r['a']} b={r['b']} excel={r['cdf']!r} model={m!r} ulps={(m - r['cdf']) / math.ulp(r['cdf']):+.0f} CR={(r['cdfcr'] - r['cdf']) / math.ulp(r['cdf']):+.0f}")
