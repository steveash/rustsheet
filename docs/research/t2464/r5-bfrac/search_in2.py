"""search.c / search2.c input from the t3571 probes: one file per run group, filtered by z and case."""
import sys, mpmath
from runs import load, groups
M = mpmath.mpf; U = M(2) ** -53
case, zsel, out = sys.argv[1], [float(v) for v in sys.argv[2].split(',')], sys.argv[3]
for key, rs in groups(load(case)).items():
    if key[2] not in zsel: continue
    with open(f'{out}_{key[0]:g}_{key[1]:g}_{key[2]:g}.txt', 'w') as f:
        f.write(f'{len(rs)}\n')
        for r in rs:
            hi = float(r['rx']); lo = float(r['rx'] - M(hi))
            ee = float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * r['y'])) / r['rx'] - 1) / U)
            f.write(' '.join(v.hex() for v in (r['a'], r['b'], r['x'], r['y'], r['pdf'], r['cdf'], hi, lo, ee)) + '\n')
