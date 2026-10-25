"""log1p implementations in IEEE double (Python floats) and x87-stored arithmetic, for ln(1-x) = log1p(-x)."""

import math
import struct

import mpmath

from lib import D, R64, mpf
from toms import alnrel_double, alnrel_x87


def cr(t):
    return float(mpmath.log1p(mpf(t)))


def kahan(t):
    u = 1.0 + t
    if u == 1.0:
        return t
    return math.log(u) * t / (u - 1.0)


def kahan2(t):
    u = 1.0 + t
    if u == 1.0:
        return t
    return math.log(u) - ((u - 1.0) - t) / u


def via_log(t):
    return math.log(1.0 + t)


# Cephes log1p (unity.c): for sqrt(1/2)-1 <= x <= sqrt(2)-1: x - x^2/2 + x^3 P(x)/Q(x)
LP = [
    4.5270000862445199635215e-5,
    4.9854102823193375972212e-1,
    6.5787325942061044846969e0,
    2.9911919328553073277375e1,
    6.0949667980987787057556e1,
    5.7112963590585538103336e1,
    2.0039553499201281259648e1,
]
LQ = [
    1.0,
    1.5062909083469192043167e1,
    8.3047565967967209469434e1,
    2.2176239823732856465394e2,
    3.0909872225312059774938e2,
    2.1642788614495947685003e2,
    6.0118660497603843919306e1,
]


def polevl(x, c):
    r = c[0]
    for v in c[1:]:
        r = r * x + v
    return r


def cephes(t):
    z = 1.0 + t
    if z < 0.70710678118654752440 or z > 1.41421356237309504880:
        return math.log(z)
    z = t * t
    z = -0.5 * z + t * (z * polevl(t, LP) / polevl(t, LQ))
    return t + z


# fdlibm s_log1p.c
Lp = [
    6.666666666666735130e-01,
    3.999999999940941908e-01,
    2.857142874366239149e-01,
    2.222219843214978396e-01,
    1.818357216161805012e-01,
    1.531383769920937332e-01,
    1.479819860511658591e-01,
]
ln2_hi, ln2_lo = 6.93147180369123816490e-01, 1.90821492927058770002e-10


def _hi(x):
    return struct.unpack("<q", struct.pack("<d", x))[0] >> 32


def fdlibm(x):
    hx = _hi(x) & 0xFFFFFFFF
    if hx >= 0x80000000:
        hx -= 1 << 32
    ax = hx & 0x7FFFFFFF
    k = 1
    if hx < 0x3FDA827A:  # x < 0.41422
        if ax < 0x3E200000:
            if 2.0**54 + x > 1e300 and ax < 0x3C900000:
                return x
            return x - x * x * 0.5
        if hx > 0 or hx <= -0x402D413D:  # -0.2929 < x < 0.41422
            k = 0
            f = x
            hu = 1
    if k != 0:
        if hx < 0x43400000:
            u = 1.0 + x
            hu = _hi(u)
            k = (hu >> 20) - 1023
            c = (1.0 - (u - x)) if k > 0 else (x - (u - 1.0))
            c /= u
        else:
            u = x
            hu = _hi(u)
            k = (hu >> 20) - 1023
            c = 0
        hu &= 0x000FFFFF
        if hu < 0x6A09E:
            u = struct.unpack(
                "<d",
                struct.pack(
                    "<q",
                    ((hu | 0x3FF00000) << 32) | (struct.unpack("<q", struct.pack("<d", u))[0] & 0xFFFFFFFF),
                ),
            )[0]
        else:
            k += 1
            u = struct.unpack(
                "<d",
                struct.pack(
                    "<q",
                    ((hu | 0x3FE00000) << 32) | (struct.unpack("<q", struct.pack("<d", u))[0] & 0xFFFFFFFF),
                ),
            )[0]
            hu = (0x00100000 - hu) >> 2
        f = u - 1.0
    hfsq = 0.5 * f * f
    if hu == 0:
        if f == 0.0:
            return 0.0 if k == 0 else k * ln2_hi + (c + k * ln2_lo)
        R = hfsq * (1.0 - 0.66666666666666666 * f)
        return f - R if k == 0 else k * ln2_hi - ((R - (k * ln2_lo + c)) - f)
    s = f / (2.0 + f)
    z = s * s
    R = z * (Lp[0] + z * (Lp[1] + z * (Lp[2] + z * (Lp[3] + z * (Lp[4] + z * (Lp[5] + z * Lp[6]))))))
    if k == 0:
        return f - (hfsq - s * (hfsq + R))
    return k * ln2_hi - ((hfsq - (s * (hfsq + R) + (k * ln2_lo + c))) - f)


def series_atanh(t):
    """2 atanh(t/(2+t)) summed in double until negligible (the engine's rlog1 style)."""
    r = t / (2.0 + t)
    r2 = r * r
    p = r
    s = 0.0
    k = 0
    while True:
        term = p / (2 * k + 1)
        s += term
        if abs(term) <= 1e-17 * abs(s):
            break
        p *= r2
        k += 1
        if k > 200:
            break
    return 2.0 * s


def x87_fyl2xp1(t):
    return D(R64(mpf(0xB17217F7D1CF79AC) * mpf(2) ** -64 * mpmath.log1p(mpf(t)) / mpmath.ln(2)))


IMPLS = {
    "cr": cr,
    "kahan": kahan,
    "kahan2": kahan2,
    "cephes": cephes,
    "fdlibm": fdlibm,
    "alnrel_d": alnrel_double,
    "alnrel_x87": lambda t: alnrel_x87(t),
    "atanh_series": series_atanh,
    "fyl2xp1": x87_fyl2xp1,
    "log(1+t)": via_log,
}
