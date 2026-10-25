"""w1-density best model of Excel's BETA.DIST(x,a,b,FALSE) and its scores.

Model M1 (see RESULTS.md):
  lnx = LN(x) (Excel's LN, stored), ly = fl(log1p(-x)) for x <= 0.375 else LN(fl(1-x))   [TOMS 708 brcomp switch]
  tx = fl(fl(a-1) * lnx), ty = fl(fl(b-1) * ly)                                         [stored products]
  lnB = TOMS 708 betaln(a, b) with Excel's GAMMALN as gamln and Excel's LN as log; its L60 reduction product
        kept in an x87 register (extended) before the log
  E = tx + (ty - lnB) in extended precision (not stored); b == 1: E = fl(tx - lnB) (stored)
  density = Excel EXP model V6 of E
  a >= 8 or both shapes > 1 (min(a,b) > 1): not modelled (the composition differs); M1 returns the exact density there.
Scores: every pdf cell of ../beta_exact.tsv and the probe1 sheets, per region, per x zone and per hold-out half.
"""

import collections
import math
import sys

import mpmath

from forms import ingredients
from gen1 import zone
from lib import D, R64, exp_v6, fmt, ln64, load_pdf, load_probe, mpf, score, xl
from toms_betaln import GL, LN, betaln


def l60_ext(a, b, w=0.0):
    """TOMS betaln's L60 with the product z held in extended precision and log = FYL2X stored."""
    a, b = mpf(a), mpf(b)
    n = int(b - 1)
    z = mpf(1)
    for _ in range(n):
        b = b - 1
        z = R64(z * R64(b / R64(a + b)))
    lz = mpf(D(ln64(z)))
    gb = mpf(GL(float(b))) if float(b) > 0 else mpf(0)
    return R64(mpf(w) + R64(lz + R64(mpf(GL(float(a))) + R64(gb - mpf(GL(float(a + b)))))))


def lnB_model(a, b):
    lo, hi = min(a, b), max(a, b)
    if lo < 1.0 and hi <= 8.0:
        # Excel: the GAMMALN form up to max = 8 inclusive (TOMS 708 switches to algdiv at 8 itself), smaller
        # shape first as TOMS 708 betaln (a > b: 34/60 old cells against 20/60 in BETA.DIST's argument order).
        ga, gb, gab = mpf(GL(lo)), mpf(GL(hi)), mpf(GL(a + b))
        return mpf(D(ga + D(gb - gab)))
    if 1.0 <= lo <= 2.0 and 2.0 < hi < 8.0:
        return l60_ext(lo, hi)
    return mpf(betaln(a, b))


def modelled(a, b):
    return (
        a < 8 and b < 8 or min(a, b) <= 1 and max(a, b) >= 8 and min(a, b) < 8
        if False
        else (min(a, b) <= 1.0)
    )


def m1(rows):
    ing = ingredients(rows)
    out = []
    cacheB = {}
    for r, g in zip(rows, ing):
        a, b, x = r["a"], r["b"], r["x"]
        if not modelled(a, b):
            out.append(r["exact"])
            continue
        if (a, b) not in cacheB:
            cacheB[(a, b)] = lnB_model(a, b)
        L = cacheB[(a, b)]
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        tx = mpf(D(g["am1"] * g["LNd"]))
        ty = mpf(D(g["bm1"] * ly))
        if b == 1.0:
            E = mpf(D(tx - L))
        else:
            E = R64(tx + R64(ty - L))
        out.append(exp_v6(E))
    return out


def region(a, b):
    ra = "a<1" if a < 1 else ("a=1" if a == 1 else ("a<8" if a < 8 else "a>=8"))
    rb = "b<1" if b < 1 else ("b=1" if b == 1 else ("b<8" if b < 8 else ("b=8" if b == 8 else "b>8")))
    return ra + "," + rb


def report(rows, models, label, by):
    groups = collections.defaultdict(list)
    for i, r in enumerate(rows):
        groups[by(r)].append(i)
    base = [r["exact"] for r in rows]
    print(f"== {label}: n={len(rows)}  M1 {fmt(score(models, rows))}  exact-rounded {fmt(score(base, rows))}")
    for g in sorted(groups):
        idx = groups[g]
        ev = [i for i in idx if i % 2 == 0]
        od = [i for i in idx if i % 2 == 1]
        print(
            f"  {g:28s} M1 {fmt(score(models, rows, idx))} | even {score(models, rows, ev)[1]}/{len(ev)} odd "
            f"{score(models, rows, od)[1]}/{len(od)} | exact-rounded {score(base, rows, idx)[1]}"
        )


if __name__ == "__main__":
    what = sys.argv[1:] or ["old", "probe"]
    if "old" in what:
        rows = load_pdf()
        m = m1(rows)
        report(rows, m, "beta_exact pdf cells by region", lambda r: region(r["a"], r["b"]))
        report(rows, m, "beta_exact pdf cells by case", lambda r: r["case"][-24:])
    if "probe" in what:
        rows = load_probe("probe1")
        m = m1(rows)
        report(rows, m, "probe1 by sheet", lambda r: f"{r['case']} a={r['a']} b={r['b']}")
        report(rows, m, "probe1 by sheet and x zone", lambda r: f"{r['case'][-2:]} {zone(r['x'])}")
