"""T-3571: the post-freeze fresh sweep (t3571-r5-bfrac-fresh). Counts for FROZEN, M0 and CR, and the Gate A envelope
of FROZEN (decision 0188 R2: within 15 significant digits of Excel, or within 1e-9 relative of 50-digit math);
every point outside is listed."""
import mpmath
from runs import load
from score_final import table, w15, cr
import model
mpmath.mp.dps = 50
rows = load('t3571-r5-bfrac-fresh')
print(f'fresh: {len(rows)} cells')
for nm, (e, w4, w) in table(rows).items(): print(f'  {nm:14s} {e}/{w4}/{w}')
out = []
for r in rows:
    m = model.cdf(r['x'], r['a'], r['b'], r['pdf']); ex = cr(r)
    if not (w15(m, r['cdf']) or abs(m - ex) <= 1e-9 * abs(ex)): out.append((r['a'], r['b'], r['x'], m, r['cdf'], ex))
print(f'Gate A envelope: {len(rows) - len(out)} of {len(rows)} inside')
for o in out: print('  outside', o)
