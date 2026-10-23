"""Residual of Excel's implied fraction error after the lambda term (extended bfrac, double lambda): structure."""
import sys, mpmath, collections, struct
from common4 import *
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
names = [f"o{o}l{l}e{e}" for o in (0, 1) for l in (0, 1, 2, 3) for e in ('1.5e-14', '1e-15', '2.2e-16', '1e-14', '3e-15')]
res = run_cf(rows, 2, exe='c/bf%d.bin')
q = []; e = {}
for i, r in enumerate(rows):
    X = M(r['x']); rx = M(r['cdfx']) / (M(r['pdfx']) * X * (1 - X))
    q.append(float(M(r['cdf']) / (M(r['pdf']) * X * M(1 - r['x'])) / rx - 1) / U)
    for ci, nm in enumerate(names):
        if 'l3' in nm: e.setdefault(nm, []).append(float(M(str(res[i][ci])) / rx - 1) / U)
q = np.array(q)
for nm, v in e.items():
    v = np.array(v); print(nm, 'model mean %+.2f sd %.2f | q - model: mean %+.2f sd %.2f' % (v.mean(), v.std(), (q - v).mean(), (q - v).std()))
rr = q - np.array(e['o0l3e2.2e-16'])
# structure: binned means over the run, autocorrelation, low-bit classes
print('bins of 64:', ' '.join('%+.1f' % rr[i:i+64].mean() for i in range(0, len(rr), 64)))
print('autocorr lag1..4:', ' '.join('%+.2f' % np.corrcoef(rr[:-k], rr[k:])[0, 1] for k in (1, 2, 3, 4)))
man = lambda xx: struct.unpack('<Q', struct.pack('<d', xx))[0] & 7
cl = collections.defaultdict(list)
for r, v in zip(rows, rr): cl[man(r['x'])].append(v)
print('low3 classes', ' '.join(f"{k}:{np.mean(cl[k]):+.2f}" for k in range(8)))
# pdf error of Excel itself
pe = np.array([float((M(r['pdf']) - M(r['pdfx'])) / M(r['pdfx'])) / U for r in rows])
print('corr(resid, Excel pdf err) %+.3f ; pdf err sd %.2f' % (np.corrcoef(rr, pe)[0, 1], pe.std()))
np.save('resid_%s.npy' % blk.replace(':', '_'), rr)
