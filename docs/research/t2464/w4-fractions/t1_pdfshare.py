"""Test 1: cdf = assembly(Excel's pdf, x, y, CF)/a for every CF variant x mode x order, per block.
Scored on cells with both Excel pdf and cdf, direct branch."""
import sys, collections
from common4 import *
rows = load(lambda d: d['branch'] == 'direct' and not math.isnan(d['pdf']) and not math.isnan(d['cdf']))
print('rows', len(rows))
blocks = collections.defaultdict(list)
for i, r in enumerate(rows): blocks[r['blk'] if r['src'] == 'w2' else 'old'].append(i)
res = {m: run_cf(rows, m) for m in (0, 1, 2)}
def orders(r, h, cfname, P):
    xx, a = r['x'], r['a']; y = 1 - xx
    out = {}
    if cfname.startswith('incbd'):
        D = h * LD(y) if False else h   # D
        out['((P*x)*D)/a'] = x('/', x('*', x('*', P, xx), D), a)
        out['(P*x)*(D/a)'] = x('*', x('*', P, xx), x('/', D, a))
        out['ext'] = float(LD(P) * LD(xx) * LD(D) / LD(a))
        return out
    if cfname.startswith('bfrac'):
        out['((P*x)*y)*r'] = x('*', x('*', x('*', P, xx), y), h)
        out['(P*(x*y))*r'] = x('*', x('*', P, x('*', xx, y)), h)
        out['ext'] = float(LD(P) * LD(xx) * LD(y) * LD(h))
        return out
    out['(h/a)*((P*x)*y)'] = x('*', x('/', h, a), x('*', x('*', P, xx), y))
    out['(((P*x)*y)*h)/a'] = x('/', x('*', x('*', x('*', P, xx), y), h), a)
    out['(P*(x*y))*(h/a)'] = x('*', x('*', P, x('*', xx, y)), x('/', h, a))
    out['((P*x)*(h/a))*y'] = x('*', x('*', x('*', P, xx), x('/', h, a)), y)
    out['ext'] = float(LD(P) * LD(xx) * LD(y) * LD(h) / LD(a))
    return out
tot = collections.Counter()
for blk, idx in sorted(blocks.items()):
    cnt = collections.Counter(); n = len(idx)
    for m in (0, 1, 2):
        for ci, cn in enumerate(CF_NAMES):
            for i in idx:
                r = rows[i]; h = res[m][i][ci]
                for pn, P in (('pdfE', r['pdf']), ('pdfCR', r['pdfcr'])):
                    for on, v in orders(r, h, cn, P).items():
                        cnt[(m, cn, pn, on)] += v == r['cdf']
    cr = sum(rows[i]['cdfcr'] == rows[i]['cdf'] for i in idx)
    a, b = rows[idx[0]]['a'], rows[idx[0]]['b']
    best = cnt.most_common(4)
    print(f"{blk:12s} a={a:<8g} b={b:<8g} n={n:5d} CR={cr:4d} | " + ' ; '.join(f"{k}:{v}" for k, v in best))
    for k, v in cnt.items(): tot[k] += v
print('TOTAL', tot.most_common(10))
