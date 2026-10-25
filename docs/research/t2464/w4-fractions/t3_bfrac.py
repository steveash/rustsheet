"""Test 3: TOMS bfrac variants against Excel's implied fraction q = cdf_e*a... at blocks where the cdf shares the
pdf.  For each variant: corr with q, residual sd, and exact counts of cdf = fl(brc*r) for brc = fl(fl(pdf_e*x)*y),
fl(pdf_e*fl(x*y)) (x87-stored or SSE)."""
import sys, collections, mpmath
from common4 import *
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
names = [f"o{o}l{l}e{e}" for o in (0, 1) for l in (0, 1, 2, 3) for e in ('1.5e-14', '1e-15', '2.2e-16', '1e-14', '3e-15')]
q = []; rx = []
for r in rows:
    X, A = M(r['x']), M(r['a'])
    I = M(r['cdfx']); P = M(r['pdfx'])
    rx.append(I / (P * X * (1 - X)))
    q.append(float(M(r['cdf']) / (M(r['pdf']) * X * M(1 - r['x']))))
q = np.array([float(M(v) / t - 1) / U for v, t in zip(q, rx)])
print(blk, len(rows), rows[0]['a'], rows[0]['b'], 'q sd %.2f mean %.2f' % (q.std(), q.mean()))
for mode in (0, 1, 2, 3, 4):
    res = run_cf(rows, mode, exe='c/bf%d.bin')
    for ci, nm in enumerate(names):
        e = np.array([float(M(str(h[ci])) / t - 1) / U for h, t in zip(res, rx)])
        cc = np.corrcoef(q, e)[0, 1] if e.std() > 0 else 0
        ex = collections.Counter()
        for r, h in zip(rows, res):
            rr = h[ci]; xx = r['x']; y = 1 - xx; P = r['pdf']
            ex['x87 (P*x)*y'] += x('*', x('*', x('*', P, xx), y), rr) == r['cdf']
            ex['x87 P*(x*y)'] += x('*', x('*', P, x('*', xx, y)), rr) == r['cdf']
            ex['sse (P*x)*y'] += (P * xx * y) * float(rr) == r['cdf']
        print(f"m{mode} {nm:16s} sd {e.std():5.2f} corr {cc:+.3f} resid {np.std(q - e):5.2f} exact " + ' '.join(f"{k}:{v}" for k, v in ex.items()))
