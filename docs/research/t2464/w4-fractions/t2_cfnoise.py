"""Test 2 at blocks where the cdf shares the pdf: Excel's implied CF error  q = cdf_e/(pdf_e*x*y/a)/CF_exact - 1
against each CF variant's own error, per block.  Reports sd(q), corr and residual sd.  Units 2^-53."""
import sys, collections, mpmath
from common4 import *
mpmath.mp.dps = 40
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
M = mpmath.mpf
U = 2.0 ** -53
q = []; cfx = []
for r in rows:
    X, A, B = M(r['x']), M(r['a']), M(r['b'])
    I = M(r['cdfx']); P = M(r['pdfx'])
    cf = I * A / (P * X * (1 - X))         # exact CF h
    cfx.append(cf)
    impl = M(r['cdf']) / (M(r['pdf']) * X * M(1 - r['x'])) * A
    q.append(float(impl / cf - 1) / U)
q = np.array(q)
print(blk, 'n', len(rows), 'a b', rows[0]['a'], rows[0]['b'], 'sd q %.2f mean %.2f' % (q.std(), q.mean()))
for m in (0, 1, 2):
    res = run_cf(rows, m)
    for ci, cn in enumerate(CF_NAMES):
        e = []
        for r, h, cf in zip(rows, res, cfx):
            v = M(float(h[ci])) if m != 2 else M(str(h[ci]))
            if cn.startswith('incbd'): v = v / M(1 - r['x'])
            if cn.startswith('bfrac'): v = v * M(r['a'])
            e.append(float(v / cf - 1) / U)
        e = np.array(e)
        if e.std() == 0: print(f"  m{m} {cn:14s} const {e.mean():+.2f}"); continue
        cc = np.corrcoef(q, e)[0, 1]
        print(f"  m{m} {cn:14s} mean {e.mean():+7.2f} sd {e.std():6.2f} corr {cc:+.3f} resid sd {np.std(q - e):.2f}")
