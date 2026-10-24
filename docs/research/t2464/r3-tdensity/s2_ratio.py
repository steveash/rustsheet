"""In-sample probes a1/a2: is T.DIST(t,n,FALSE) one stored product K * X (or X / K) of an inner Excel column X with a
per-df constant K?  For each cell the K giving exactly Excel's value form an interval (K * X rounded once, or the
x87 double rounding); the deepest overlap per df is reported, with the spread of B/X in units.
usage: s2_ratio.py"""
from collections import defaultdict
from fractions import Fraction

import numpy as np

import tdlib as TD

rows = TD.load_probe("a1") + TD.load_probe("a2")
bad = [r for r in rows if r["t_rb"] != r["t"]]
print("rows", len(rows), "t read-back mismatches", len(bad))
by = defaultdict(list)
for r in rows:
    by[r["n"]].append(r)


def kint(B, X, mode):
    """Interval of real K with RN(K*X) == B (mode 'mul') or RN(X/K) == B ('div'), as Fractions (half-ulp bounds)."""
    lo = Fraction(B) - Fraction(B - __import__("math").nextafter(B, 0)) / 2
    hi = Fraction(B) + Fraction(__import__("math").nextafter(B, 2 * B) - B) / 2
    X = Fraction(X)
    if mode == "mul":
        return lo / X, hi / X
    return X / hi, X / lo


for col in ("fdist", "pow1q", "pownu", "betay", "betaz", "powy"):
    out = []
    for n in sorted(by):
        rr = [r for r in by[n] if r[col] > 0]
        if len(rr) < 20:
            continue
        if col == "fdist":
            ratios = [r["excel"] / (r[col] * abs(r["t"])) for r in rr]
            ivs = [kint(r["excel"], Fraction(r[col]) * Fraction(abs(r["t"])), "mul") for r in rr]
        else:
            ratios = [r["excel"] / r[col] for r in rr]
            ivs = [kint(r["excel"], r[col], "mul") for r in rr]
        ratios = np.array(ratios)
        sd = ratios.std() / ratios.mean() / 2**-53
        ev = sorted([(a, 0) for a, b in ivs] + [(b, 1) for a, b in ivs])
        c = best = 0
        for p, t in ev:
            c += 1 if t == 0 else -1
            best = max(best, c)
        out.append(f"n={n:g}: {best}/{len(rr)} sd {sd:.1f}u")
    print(col, "|", "; ".join(out))
