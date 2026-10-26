"""Loader-form pdf variants (bd0 series/direct/log1p, lf old/new, exp(log(n+1)+l) vs (n+1)*exp(l)) vs Excel pdf."""
import sys, subprocess, mpmath, collections
from common4 import *
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blks = sys.argv[1:] or ['xruns-1:7']
for blk in blks:
    rows = load(lambda d: d['blk'] == blk and not math.isnan(d['pdf']))
    pe = np.array([float((M(r['pdf']) - M(r['pdfx'])) / M(r['pdfx'])) / U for r in rows])
    inp = ''.join(f"{r['a'].hex()} {r['b'].hex()} {r['x'].hex()} {(1 - r['x']).hex()}\n" for r in rows)
    for exe in ('pdfv_sse', 'pdfv_387'):
        out = subprocess.run(['ext/r36/%s.bin' % exe], input=inp, capture_output=True, text=True).stdout.split('\n')
        vals = [[float.fromhex(t) for t in l.split()] for l in out[:len(rows)]]
        for vi in range(12):
            e = np.array([float((M(v[vi]) - M(r['pdfx'])) / M(r['pdfx'])) / U for v, r in zip(vals, rows)])
            ex = sum(v[vi] == r['pdf'] for v, r in zip(vals, rows))
            print(blk, exe, f"bd0{'sdl'[vi // 2 % 3]} lf{'new' if vi < 6 else 'old'} {'explog' if vi % 2 == 0 else 'mul'}: exact {ex}/{len(rows)} sd {e.std():.2f} corr {np.corrcoef(e, pe)[0,1]:+.3f} resid {np.std(pe - e):.2f}")
    print(blk, 'Excel pdf err sd %.2f, CR exact %d' % (pe.std(), sum(r['pdfcr'] == r['pdf'] for r in rows)))
