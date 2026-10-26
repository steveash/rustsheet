"""TOMS-X bfrac region: cdf = fl(fl(fl(pdf*x)*y)*r) and fl(pdf*fl(x*y))*r with r = TOMS bfrac fraction in double
(lambda = fl(a - fl((a+b)x)), or the a>b TOMS form), pdf = correctly rounded density or Excel's own pdf cell when
the data has it.  Scored on old direct cells on the bfrac path and on w2's (100,120) run."""
import collections
from common4 import *
import tomsx
rows = load(lambda d: d['branch'] == 'direct' and not math.isnan(d['cdf']) and d['a'] > 1 and d['b'] > 1)
rows = [r for r in rows if tomsx.toms_path(r['x'], r['a'], r['b']) == 'bfrac']
names = [f"o{o}l{l}e{e}" for o in (0, 1) for l in (0, 1, 2, 3) for e in ('1.5e-14', '1e-15', '2.2e-16', '1e-14', '3e-15')]
res = run_cf(rows, 0, exe='c/bf%d.bin')
for src in ('old', 'w2'):
    idx = [i for i, r in enumerate(rows) if r['src'] == src]
    cnt = collections.Counter()
    for i in idx:
        r = rows[i]; xx = r['x']; y = 1 - xx
        for ci in (names.index('o0l2e1.5e-14'), names.index('o1l2e1.5e-14'), names.index('o0l2e2.2e-16')):
            rr = float(res[i][ci])
            for pn, P in (('pdfCR', r['pdfcr']), ('pdfE', r['pdf'])):
                if math.isnan(P): continue
                cnt[(names[ci], pn, '(Px)y')] += (P * xx * y) * rr == r['cdf']
                cnt[(names[ci], pn, 'P(xy)')] += (P * (xx * y)) * rr == r['cdf']
    cr = sum(rows[i]['cdfcr'] == rows[i]['cdf'] for i in idx)
    print(src, 'n', len(idx), 'CR exact', cr, 'shapes', sorted({(rows[i]['a'], rows[i]['b']) for i in idx})[:12])
    for k, v in cnt.most_common(6): print('  ', k, v)
