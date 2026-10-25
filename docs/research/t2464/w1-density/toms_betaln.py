"""TOMS 708 betaln / algdiv / bcorr (R toms708.c structure) in IEEE double, with gamln and log pluggable.

gamln defaults to Excel's GAMMALN (xl), log to Excel's LN (x87 model). gsumln uses gamln(a+b) directly
(TOMS uses gamln1/alnrel; for the cases probed here a+b is a double so lnGamma(a+b) is what it approximates).
"""

import math

from lib import xl
from toms import alnrel_double

C0, C1, C2, C3, C4, C5 = (
    0.0833333333333333,
    -0.00277777777760991,
    7.9365066682539e-4,
    -5.9520293135187e-4,
    8.37308034031215e-4,
    -0.00165322962780713,
)


def GL(v):
    return xl("GAMMALN", [v])[0]


def LN(v):
    return xl("LN", [v])[0]


def alnrel(a, log=LN):
    if abs(a) > 0.375:
        return log(1.0 + a)
    return alnrel_double(a)


def _del_b(c, x, b):
    """The del(b) series shared by TOMS algdiv and bcorr (s3..s11 in powers of x, scaled by c / b)."""
    x2 = x * x
    s3 = x + x2 + 1.0
    s5 = x + x2 * s3 + 1.0
    s7 = x + x2 * s5 + 1.0
    s9 = x + x2 * s7 + 1.0
    s11 = x + x2 * s9 + 1.0
    t = 1.0 / b
    t = t * t
    w = ((((C5 * s11 * t + C4 * s9) * t + C3 * s7) * t + C2 * s5) * t + C1 * s3) * t + C0
    w *= c / b
    return w


def algdiv(a, b, log=LN):
    if a > b:
        h = b / a
        c = 1.0 / (h + 1.0)
        x = h / (h + 1.0)
        d = a + (b - 0.5)
    else:
        h = a / b
        c = h / (h + 1.0)
        x = 1.0 / (h + 1.0)
        d = b + (a - 0.5)
    w = _del_b(c, x, b)
    u = d * alnrel(a / b, log)
    v = a * (log(b) - 1.0)
    return (w - v - u) if u > v else (w - u - v)


def bcorr(a0, b0):
    a, b = min(a0, b0), max(a0, b0)
    h = a / b
    c = h / (h + 1.0)
    x = 1.0 / (h + 1.0)
    w = _del_b(c, x, b)
    t = 1.0 / a
    t = t * t
    return (((((C5 * t + C4) * t + C3) * t + C2) * t + C1) * t + C0) / a + w


def betaln(a0, b0, gamln=GL, log=LN):
    e = 0.918938533204673
    a, b = min(a0, b0), max(a0, b0)
    if a >= 8.0:
        w = bcorr(a, b)
        h = a / b
        c = h / (h + 1.0)
        u = -(a - 0.5) * log(c)
        v = b * alnrel(h, log)
        return (log(b) * -0.5 + e + w - v - u) if u > v else (log(b) * -0.5 + e + w - u - v)
    if a < 1.0:
        if b < 8.0:
            return gamln(a) + (gamln(b) - gamln(a + b))
        return gamln(a) + algdiv(a, b, log)
    if a <= 2.0:
        if b <= 2.0:
            return gamln(a) + gamln(b) - gamln(a + b)
        w = 0.0
        if b < 8.0:
            return _l60(a, b, w, gamln, log)
        return gamln(a) + algdiv(a, b, log)
    if b > 1000.0:
        n = int(a - 1.0)
        w = 1.0
        for _ in range(n):
            a -= 1.0
            w *= a / (a + b)
        return log(w) - n * log(b) + (gamln(a) + algdiv(a, b, log))
    n = int(a - 1.0)
    w = 1.0
    for _ in range(n):
        a -= 1.0
        h = a / b
        w *= h / (h + 1.0)
    w = log(w)
    if b >= 8.0:
        return w + gamln(a) + algdiv(a, b, log)
    return _l60(a, b, w, gamln, log)


def _l60(a, b, w, gamln, log):
    n = int(b - 1.0)
    z = 1.0
    for _ in range(n):
        b -= 1.0
        z *= b / (a + b)
    return w + log(z) + (gamln(a) + (gamln(b) - gamln(a + b)))


def algdiv_x87(a, b, store_final=True, logmode="ln64", alnrel_mode="x87e"):
    """algdiv with every op in x87 extended (R64), log = FYL2X extended, alnrel's rational in extended."""
    from lib import D, R64, ln64, mpf
    from toms import alnrel_x87
    A, B = mpf(a), mpf(b)
    if a > b:
        h = R64(B / A); c = R64(1 / R64(h + 1)); x = R64(h / R64(h + 1)); d = R64(A + R64(B - 0.5))
    else:
        h = R64(A / B); c = R64(h / R64(h + 1)); x = R64(1 / R64(h + 1)); d = R64(B + R64(A - 0.5))
    x2 = R64(x * x)
    s3 = R64(R64(x + x2) + 1)
    s5 = R64(R64(x + R64(x2 * s3)) + 1)
    s7 = R64(R64(x + R64(x2 * s5)) + 1)
    s9 = R64(R64(x + R64(x2 * s7)) + 1)
    s11 = R64(R64(x + R64(x2 * s9)) + 1)
    t = R64(1 / B); t = R64(t * t)
    w = R64(mpf(C5) * s11)
    w = R64(R64(R64(w * t) + R64(mpf(C4) * s9)) * t)
    w = R64(R64(w + R64(mpf(C3) * s7)) * t)
    w = R64(R64(w + R64(mpf(C2) * s5)) * t)
    w = R64(R64(w + R64(mpf(C1) * s3)) * t)
    w = R64(w + mpf(C0))
    w = R64(w * R64(c / B))
    q = R64(A / B)
    if alnrel_mode == "x87e":
        al = mpf(alnrel_x87(q, store=False)) if abs(q) <= 0.375 else R64(ln64(R64(1 + q)))
    else:
        al = mpf(alnrel(float(q)))
    lb = ln64(B) if logmode == "ln64" else mpf(D(ln64(B)))
    u = R64(d * al)
    v = R64(A * R64(lb - 1))
    r = R64(R64(w - v) - u) if u > v else R64(R64(w - u) - v)
    return D(r) if store_final else r
