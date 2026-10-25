"""Interval inversion helper: shifts d (absolute, units 2^-53) of an extended exponent E with EXP(E+d) == target."""

import mpmath

from lib import exp_v6, mpf

U = mpf(2) ** -53


def shift_interval(E, target, span=64):
    """[lo, hi] in units U of shifts d with exp_v6(E + d*U) == target, or None. Assumes monotone EXP."""
    E = mpf(E)
    d0 = (mpmath.log(mpf(target)) - E) / U
    f = lambda d: exp_v6(E + d * U)
    hit = None
    for k in range(0, 4 * span):
        for s in (1, -1):
            d = d0 + s * k / 8
            if f(d) == target:
                hit = d
                break
        if hit is not None:
            break
    if hit is None:
        return None
    lo_in, lo_out = hit, hit - 2
    while f(lo_out) == target:
        lo_out -= 2
    hi_in, hi_out = hit, hit + 2
    while f(hi_out) == target:
        hi_out += 2
    for _ in range(30):
        m = (lo_in + lo_out) / 2
        if f(m) == target:
            lo_in = m
        else:
            lo_out = m
        m = (hi_in + hi_out) / 2
        if f(m) == target:
            hi_in = m
        else:
            hi_out = m
    return float(lo_in), float(hi_in)
