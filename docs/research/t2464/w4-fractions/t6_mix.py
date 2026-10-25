"""Mixed-precision bfrac scan: corr of model internal error with Excel's implied fraction error, and exact counts with
cdf = x87(x87(x87(pdf_e*x)*y)*r)."""
import sys, subprocess, mpmath, collections
from common4 import *
mpmath.mp.dps = 40
M = mpmath.mpf; U = 2.0 ** -53
blk = sys.argv[1] if len(sys.argv) > 1 else 'xruns-1:7'
eps = sys.argv[2] if len(sys.argv) > 2 else '1.5e-14'
rows = load(lambda d: d['blk'] == blk and d['branch'] == 'direct' and not math.isnan(d['pdf']))
inp = ''.join(f"{r['a'].hex()} {r['b'].hex()} {r['x'].hex()} {(1 - r['x']).hex()}\n" for r in rows)
out = subprocess.run(['c/bfmix.bin', eps], input=inp, capture_output=True, text=True, check=True).stdout.split('\n')
res = [[parse_ld(t) for t in l.split()] for l in out[:len(rows)]]
q = []; rx = []
for r in rows:
    X = M(r['x']); v = M(r['cdfx']) / (M(r['pdfx']) * X * (1 - X)); rx.append(v)
    q.append(float(M(r['cdf']) / (M(r['pdf']) * X * M(1 - r['x'])) / v - 1) / U)
q = np.array(q)
lines = []
for ci in range(128):
    e = np.array([float(M(str(h[ci])) / v - 1) / U for h, v in zip(res, rx)])
    ex = sum(x('*', x('*', x('*', r['pdf'], r['x']), 1 - r['x']), float(h[ci])) == r['cdf'] for r, h in zip(rows, res))
    ex2 = sum(x('*', x('*', r['pdf'], x('*', r['x'], 1 - r['x'])), float(h[ci])) == r['cdf'] for r, h in zip(rows, res))
    cc = np.corrcoef(q, e)[0, 1] if e.std() > 0 else 0
    lines.append((np.std(q - e), f"ord{ci // 64} mask{ci % 64:2d} sd {e.std():5.2f} corr {cc:+.3f} resid {np.std(q - e):5.2f} meanres {np.mean(q - e):+.2f} exact {ex} / {ex2}"))
lines.sort()
print(blk, len(rows), 'eps', eps, 'q sd %.2f' % q.std())
for l in lines[:12]: print(l[1])
