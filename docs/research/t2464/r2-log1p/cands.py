"""log1p(t) candidates (t = -p), all returning a double. Mostly imported from ../w1-density (log1p_impls, toms)."""
import math

import mpmath

import r2lib as R  # noqa: F401  (sets sys.path)
import log1p_impls as W1  # noqa: E402
from lib import D, R64, mpf, ln_x87  # noqa: E402

IMPLS = dict(W1.IMPLS)


def ln_fl(t):
    """Excel's LN(fl(1+t)) (x87 FYL2X model)."""
    return ln_x87(1.0 + t)


IMPLS["LN(fl(1+t))"] = ln_fl


# --- "ideal after r" family: r = stored a/(a+2), then an exact (or nearly exact) 2 atanh(r)
def r_store(a, mode):
    """t = a/(a+2) with the given storage: 'd' IEEE double, 'x' x87 stored ops, 'e' extended (no stores),
    'ed' extended sum then stored quotient, 'exact'."""
    A = mpf(a)
    if mode == "d":
        return mpf(a / (a + 2.0))
    if mode == "x":
        return mpf(D(A / D(A + 2)))
    if mode == "e":
        return R64(A / R64(A + 2))
    if mode == "xd":  # sum stored to double, quotient kept extended
        return R64(A / D(A + 2))
    if mode == "exact":
        return A / (A + 2)
    raise ValueError(mode)


def ideal(mode):
    return lambda a: float(2 * mpmath.atanh(r_store(a, mode)))


for _m in ("d", "x", "e", "xd"):
    IMPLS[f"ideal2atanh_{_m}"] = ideal(_m)


def via_u(mode):
    """log1p as log(u) - (u-1-a)/u correction with u = fl(1+a) and the log/LN exact then rounded (mode 'cr') ."""
    def f(a):
        u = 1.0 + a
        if u == 1.0:
            return a
        return float(mpmath.log(mpf(u)) - (mpf(u) - 1 - mpf(a)) / mpf(u))
    return f


IMPLS["ideal_u_corr"] = via_u("cr")
