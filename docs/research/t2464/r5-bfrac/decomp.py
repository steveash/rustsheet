"""Excel's implied fraction error vs the exact r (exact_r.py) and vs double bfrac variants (bfrac_fma.c)."""
import subprocess, statistics as st, numpy as np, mpmath
from exact_r import get
rows = get()
inp = ''.join(f"{r['a'].hex()} {r['b'].hex()} {r['x'].hex()} {(1 - r['x']).hex()}\n" for r in rows)
out = subprocess.run(['/tmp/claude-0/bffma'], input=inp, capture_output=True, text=True, check=True).stdout.split('\n')
U = mpmath.mpf(2) ** -53
M = mpmath.mpf
ee = np.array([float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * (1 - r['x']))) / r['rx'] - 1) / U) for r in rows])
ed = np.array([float((M(float.fromhex(l.split()[0])) / r['rx'] - 1) / U) for r, l in zip(rows, out)])
ed2 = np.array([float((M(float.fromhex(l.split()[2])) / r['rx'] - 1) / U) for r, l in zip(rows, out)])
# rounding of the assembly fl(P*fl(xy)) relative
asm = np.array([float((M(r['pdf'] * (r['x'] * (1 - r['x']))) / (M(r['pdf']) * M(r['x']) * M(1 - r['x'])) - 1) / U) for r in rows])
print('Excel implied fraction error (incl. assembly/final rounding): mean %+.2f sd %.2f' % (ee.mean(), ee.std()))
print('double bfrac eps1.5e-14: mean %+.2f sd %.2f; corr with Excel %+.3f' % (ed.mean(), ed.std(), np.corrcoef(ee, ed)[0, 1]))
print('double bfrac eps2.2e-16: mean %+.2f sd %.2f; corr with Excel %+.3f' % (ed2.mean(), ed2.std(), np.corrcoef(ee, ed2)[0, 1]))
print('assembly rounding fl(P*fl(xy)) sd %.2f, corr with Excel %+.3f' % (asm.std(), np.corrcoef(ee, asm)[0, 1]))
np.save('/tmp/claude-0/ee.npy', ee)
