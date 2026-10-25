"""Recompute exact cdf and pdf (mpmath dps 50) for every row of data.tsv -> data.tsv (in place, cdfx/pdfx replaced),
and report the discrepancy against the inherited values (w2's exact_w2f.tsv had errors up to several 2^-53)."""
import csv, mpmath
from multiprocessing import Pool
import numpy as np
def work(t):
    mpmath.mp.dps = 50
    x, a, b = (mpmath.mpf(float(v)) for v in t)
    I = mpmath.betainc(a, b, 0, x, regularized=True)
    P = mpmath.exp((a - 1) * mpmath.log(x) + (b - 1) * mpmath.log1p(-x) - mpmath.log(mpmath.beta(a, b)))
    return mpmath.nstr(I, 35), mpmath.nstr(P, 35)
if __name__ == '__main__':
    rows = list(csv.DictReader(open('data.tsv'), delimiter='\t'))
    with Pool(8) as p:
        out = p.map(work, [(r['x'], r['a'], r['b']) for r in rows], chunksize=100)
    mpmath.mp.dps = 50
    dc = {}; 
    for r, (I, P) in zip(rows, out):
        for k, new in (('cdfx', I), ('pdfx', P)):
            old = mpmath.mpf(r[k]); nw = mpmath.mpf(new)
            dc.setdefault((r['src'], k), []).append(float(abs(old - nw) / nw) / 2**-53)
            r[k] = new
    for k, v in dc.items():
        v = np.array(v); print(k, 'n', len(v), 'median %.3f  p90 %.3f max %.2f  >0.5: %d' % (np.median(v), np.percentile(v, 90), v.max(), (v > 0.5).sum()))
    with open('data.tsv', 'w') as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]), delimiter='\t'); w.writeheader(); w.writerows(rows)
