"""Interval inversion: per cell, the set of exponent shifts d (E -> E + d, absolute) for which EXP(E + d) equals
Excel's density, with E the leader's exponent tx + (ty - lnB). Intersect across the x values of one (case, a, b):
a shared-B model needs a non-empty intersection. Then list d per (a, b) in units of ulp(lnB) and check whether
lnB - d can be a double (the shift is a different stored lnB).

Usage: interval1.py [region] (default a<1,b<=8).
"""

import collections
import math
import sys

import mpmath

from lib import pdf_region, D, R64, exp_v6, ln64, load_pdf, mpf, xl

rows, X, A, Bv, AB, (GLa, GLb, GLab), want, idx = pdf_region()
N = len(rows)
U = mpf(2) ** -53


def E_parts(i):
    x, a, b = mpf(X[i]), mpf(A[i]), mpf(Bv[i])
    lnB = mpf(D(mpf(GLa[i]) + D(mpf(GLb[i]) - GLab[i])))
    tx = mpf(D(D(a - 1) * D(ln64(x))))
    ty = R64(D(b - 1) * ln64(1 - x))
    return tx, ty, lnB


def interval(i, E):
    """[lo, hi] of shifts d with exp_v6(E + d) == excel (in units U), by bisection on each side."""
    target = rows[i]["excel"]
    # find a shift that hits: start from log ratio
    d0 = mpmath.log(mpf(target)) - E
    f = lambda d: exp_v6(E + d)
    if f(d0) != target:
        # scan nearby
        for k in range(1, 40):
            for s in (1, -1):
                if f(d0 + s * k * U / 8) == target:
                    d0 = d0 + s * k * U / 8
                    break
            else:
                continue
            break
        else:
            return None
    lo_in, lo_out = d0, d0 - 4 * U
    while f(lo_out) == target:
        lo_out -= 4 * U
    for _ in range(40):
        m = (lo_in + lo_out) / 2
        if f(m) == target:
            lo_in = m
        else:
            lo_out = m
    hi_in, hi_out = d0, d0 + 4 * U
    while f(hi_out) == target:
        hi_out += 4 * U
    for _ in range(40):
        m = (hi_in + hi_out) / 2
        if f(m) == target:
            hi_in = m
        else:
            hi_out = m
    return float(lo_in / U), float(hi_in / U)


groups = collections.defaultdict(list)
for i in idx:
    tx, ty, lnB = E_parts(i)
    E = R64(tx + R64(ty - lnB))
    iv = interval(i, E)
    groups[(rows[i]["case"], A[i], Bv[i])].append((i, iv, lnB))
stats = collections.Counter()
out = []
for key, g in sorted(groups.items()):
    ivs = [iv for _, iv, _ in g if iv]
    if not ivs:
        continue
    lo = max(v[0] for v in ivs)
    hi = min(v[1] for v in ivs)
    lnB = g[0][2]
    ul = math.ulp(float(lnB)) / float(U)
    contains0 = lo <= 0 <= hi
    # E + d = tx + ty - (lnB - d): candidate stored lnB' = lnB - d; doubles k*ulp offsets
    ks = [k for k in range(-6, 7) if lo <= -k * ul * -1 <= hi]  # d = k*ul means lnB' = lnB - k ulp
    ks = [k for k in range(-6, 7) if lo <= k * ul <= hi]
    stats["groups"] += 1
    stats["nonempty"] += lo <= hi
    stats["contains0"] += contains0
    stats["has lnB double"] += bool(ks)
    out.append((key, len(ivs), lo, hi, ul, ks))
print(want, dict(stats))
for key, n, lo, hi, ul, ks in out:
    print(
        f"{key[0][-12:]} a={key[1]} b={key[2]} n={n} d in [{lo:+.3f},{hi:+.3f}] ulp(lnB)={ul:.2f} lnB-shift k={ks}"
    )
