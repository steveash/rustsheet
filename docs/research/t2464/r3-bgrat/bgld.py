"""T-2512 r3-bgrat: TOMS bgrat for general b in x87 extended (r2-tdist's bgrat_half_ld generalised) on Excel's
direct bgrat cells (probe t2512-r3-bgrat-split).

Stored doubles as at b = 1/2: lnx (Excel's LN / log1p, FINAL's cut) and z = fl(-nu lnx); everything else numpy
longdouble, the result rounded once. Q(b, z) for the series start: exact (mpmath, to longdouble) or TOMS grat1's
continued fraction in longdouble (all z here are >= 1.1) at eps. Stop rule |dj| <= eps (s + l) at eps 15e-15 (TOMS),
1e-15 and 1e-17. r = z^b e^-z / Gamma(b) (exact Gamma; u * j0 = E * Q, so r cancels in the leading term)."""
from __future__ import annotations

from collections import defaultdict

import mpmath
import numpy as np

import bscore as score
from common import split_cells, table, xlk

LD = np.longdouble
C = {(c["a"], c["b"], c["x"]): c for c in score.cells()}


def q_cf(b, x, r, eps):
    """TOMS grat1's continued fraction for Q(b, x), x >= 1.1, in longdouble (r = x^b e^-x / Gamma(b))."""
    a2nm1, a2n, b2nm1, b2n, c = LD(1), LD(1), x, x + (1 - b), LD(1)
    for _ in range(2000):
        a2nm1 = x * a2n + c * a2nm1
        b2nm1 = x * b2n + c * b2nm1
        am0 = a2nm1 / b2nm1
        c += 1
        cma = c - b
        a2n = a2nm1 + cma * a2n
        b2n = b2nm1 + cma * b2n
        an0 = a2n / b2n
        if abs(an0 - am0) < LD(eps) * an0:
            break
    return r * an0


def bgrat_ld(a, b, lnx, qmode, eps):
    bL = LD(b)
    nu = LD(a) + LD(0.5) * (bL - 1)
    zd = float(-nu * LD(lnx))
    z, L = LD(zd), LD(lnx)
    with mpmath.workdps(30):
        rg = LD(float(1 / mpmath.gamma(b)))
        qx = LD(float(mpmath.gammainc(b, zd, mpmath.inf, regularized=True)))
    r = np.exp(bL * np.log(z) - z) * rg
    E = np.exp(-(xlk._algdiv_ld(b, a) + bL * np.log(nu)))
    u = r * E
    q = qx if qmode == "exact" else q_cf(bL, z, r, eps)
    j = q / r
    s = j
    v = LD(0.25) / (nu * nu)
    t2 = LD(0.25) * L * L
    t, cn, n2 = LD(1), LD(1), LD(0)
    cs, ds = [], []
    for n in range(1, 31):
        bp2n = bL + n2
        j = (bp2n * (bp2n + 1) * j + (z + bp2n + 1) * t) * v
        n2 += 2
        t *= t2
        cn /= n2 * (n2 + 1)
        cs.append(cn)
        acc, coef = LD(0), bL - n
        for i in range(1, n):
            acc += coef * cs[i - 1] * ds[n - 1 - i]
            coef += bL
        ds.append((bL - 1) * cn + acc / n)
        dj = ds[-1] * j
        s += dj
        if s <= 0 or abs(dj) <= LD(eps) * s:
            break
    return float(u * s)


g = defaultdict(list)
for a, b0, x, _case, d in split_cells():
    c = C[(a, b0, x)]
    half = "hold" if c["row"]["hold"] else "tune"
    for qm in ("exact", "cf"):
        for e in (15e-15, 1e-15, 1e-17):
            m = bgrat_ld(a, b0, c["lnx"], qm, e)
            g[(f"Q {qm}, eps {e:g}", half)].append((m, d))
            g[(f"Q {qm}, eps {e:g}", "ALL")].append((m, d))
table(g, ("model", "half"))
