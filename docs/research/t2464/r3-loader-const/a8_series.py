"""a8: does Excel's stirlerr series for v > 15 use R's term counts (5 terms to 35, 4 to 80, 3 to 500, 2 above)?

For each shape with a clean X fit (within-shape sd < 1.2 units), the residual r = needed offset - M3's constant
offset (exact stirlerr) is compared with the change a truncated series makes: delta = -(series_T(v) - exact(v)) for
v = k, nk (and + for n), summed, where T(v) is the number of terms used at v. Thresholds are scanned.
Usage: a8_series.py
"""

from __future__ import annotations

import csv
import itertools
import math

import mpmath

import a3_const as A3
import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
S = [mpf(1) / 12, mpf(1) / 360, mpf(1) / 1260, mpf(1) / 1680, mpf(1) / 1188]


def series_exact(v, terms):
    v = mpf(v)
    tot = mpf(0)
    for j in range(terms):
        tot += (-1) ** j * S[j] / v ** (2 * j + 1)
    return tot


def delta(v, thr):
    """series(v) - exact stirlerr(v) for v > 15 under thresholds thr = (t5, t4, t3): 5 terms up to t5, 4 up to t4, 3
    up to t3, 2 above."""
    v = mpf(v)
    if v <= 15:
        return mpf(0)
    t5, t4, t3 = thr
    terms = 5 if v <= t5 else 4 if v <= t4 else 3 if v <= t3 else 2
    return series_exact(v, terms) - A3.st_exact(v)


def main():
    rows = []
    for w in ("sweep", "diverse"):
        for r in csv.DictReader(open(R.HERE / f"off_{w}_R.tsv"), delimiter="\t"):
            if float(r["within_sd"]) >= 1.2:
                continue
            a, b = float(r["a"]), float(r["b"])
            p = float((A3.cand(a, b, ("exact", "exact"), "LF0", "e", "xl") - R.exact_const(a, b)) / U)
            k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
            if max(k, n, nk) <= 15:
                continue
            rows.append((float(r["offset_units"]) - p, k, n, nk))
    print(f"{len(rows)} clean shapes with a piece > 15")
    grid = [15, 20, 25, 30, 35, 40, 50, 60, 80, 100, 150, 200, 500, 1e9]
    res = []
    for t5, t4, t3 in itertools.product(grid, grid, grid):
        if not (t5 <= t4 <= t3):
            continue
        thr = (t5, t4, t3)
        d = [o - float((delta(n, thr) - delta(k, thr) - delta(nk, thr)) / U) for o, k, n, nk in rows]
        rms = math.sqrt(sum(v * v for v in d) / len(d))
        res.append((rms, thr, sum(abs(v) <= 2 for v in d)))
    res.sort()
    for rms, thr, c in res[:12]:
        print(f"  thresholds 5-term<= {thr[0]:g}, 4-term<= {thr[1]:g}, 3-term<= {thr[2]:g}: rms {rms:.2f}, |d|<=2 {c}/{len(rows)}")
    print("  fine scan of the 4-term/3-term cut (5 terms to 35, 2 terms above 500):")
    for t4 in range(36, 82, 2):
        thr = (35, t4, 500)
        d = [o - float((delta(n, thr) - delta(k, thr) - delta(nk, thr)) / U) for o, k, n, nk in rows]
        print(f"    t4 {t4}: rms {math.sqrt(sum(v * v for v in d) / len(d)):.3f}  |d|<=2 {sum(abs(v) <= 2 for v in d)}")
    R0 = (35, 80, 500)
    d = [o - float((delta(n, R0) - delta(k, R0) - delta(nk, R0)) / U) for o, k, n, nk in rows]
    print(f"  R's (35, 80, 500): rms {math.sqrt(sum(v * v for v in d) / len(d)):.2f}")


if __name__ == "__main__":
    main()
