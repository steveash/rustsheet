"""Truncation bias of bfrac's fraction after N terms (exact arithmetic) against Excel's mean implied fraction error,
and the TOMS stop test |r - r0| <= eps*r: the step size at each N in units of r. Pins how many terms Excel takes."""
import numpy as np, mpmath
from exact_r import get
mpmath.mp.dps = 40
M = mpmath.mpf; U = M(2) ** -53
rows = get()
ee = np.array([float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * (1 - r['x']))) / r['rx'] - 1) / U) for r in rows])
def conv(a, b, x, y, lam, N):
    a, b, x, y, lam = map(M, (a, b, x, y, lam))
    c = 1 + lam; c0 = b / a; c1 = 1 + 1 / a; yp1 = y + 1; n = M(0); p = M(1); s = a + 1
    an, bn, anp1, bnp1 = M(0), M(1), M(1), c / c1; out = [c1 / c]
    for _ in range(N):
        n += 1; t = n / a; w = n * (b - n) * x; e = a / s
        al = p * (p + c0) * e * e * (w * x); e = (1 + t) / (c1 + t + t); be = n + w / s + e * (c + n * yp1)
        p = 1 + t; s += 2
        an, anp1 = anp1, al * an + be * anp1; bn, bnp1 = bnp1, al * bn + be * bnp1
        out.append(anp1 / bnp1)
    return out
for r in rows[::256]:
    y = 1 - r['x']; lam = r['a'] - (r['a'] + r['b']) * r['x']
    cv = conv(r['a'], r['b'], r['x'], y, lam, 24)
    print('x', r['x'].hex(), ' '.join(f"N{N}:{float((cv[N]/r['rx']-1)/U):+.2f}/step{float(abs(cv[N]-cv[N-1])/cv[N]):.1e}" for N in range(14, 22)))
print('Excel mean implied error %+.2f (se %.2f)' % (ee.mean(), ee.std() / len(ee) ** 0.5))
