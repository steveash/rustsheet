"""Score FMA-contraction variants of bfrac (bfrac_fma.c) on Excel's (100,120) run: cdf = fl(fl(P*fl(x*y))*r), P = Excel's
own pdf cell. Prints exact count and the residual (Excel's implied r minus model r, units of 2^-53 relative)."""
import subprocess, sys, statistics
from fractions import Fraction as F
from data import load
EXE = sys.argv[1] if len(sys.argv) > 1 else '/tmp/claude-0/bffma'
case, blk = (sys.argv[2], int(sys.argv[3])) if len(sys.argv) > 3 else ('t2464-w2f-xruns-1', 7)
rows = load(case, blk)
inp = ''.join(f"{r['a'].hex()} {r['b'].hex()} {r['x'].hex()} {(1 - r['x']).hex()}\n" for r in rows)
out = subprocess.run([EXE], input=inp, capture_output=True, text=True, check=True).stdout.split('\n')
masks = [m for m in range(128) if (m & 12) != 12]
U = F(1, 2 ** 53)
imp = [F(r['cdf']) / F(r['pdf'] * (r['x'] * (1 - r['x']))) for r in rows]
res = []
for mi, m in enumerate(masks):
    for ei, e in enumerate(('1.5e-14', '1e-15', '2.2e-16')):
        ex = 0; d = []
        for r, line, ip in zip(rows, out, imp):
            rv = float.fromhex(line.split()[3 * mi + ei])
            cdf = (r['pdf'] * (r['x'] * (1 - r['x']))) * rv
            ex += cdf == r['cdf']
            d.append(float((ip - F(rv)) / F(rv) / U))
        res.append((ex, m, e, statistics.mean(d), statistics.pstdev(d)))
res.sort(reverse=True)
print('n', len(rows))
for ex, m, e, mu, sd in res[:25]: print(f'mask {m:3d} eps {e:8s} exact {ex:4d}  resid mean {mu:+.2f} sd {sd:.2f}')
base = [x for x in res if x[1] == 0]
for x in base: print('base', x)
