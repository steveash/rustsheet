"""Exhaustive left-to-right and pairwise sums of the exponent terms, each op stored (d) or extended (e).

Term sets: S3 = [tx, ty, -L1d]; S4 = [tx, ty, -ga, -(gb-gab)]; S5 = [tx, ty, -ga, -gb, +gab].
tx = D(am1*LN(x)), ty = D(bm1*ly) with brcomp logs. Reports the best patterns per x zone.
Usage: sum4.py sheet|old [S3|S4|S5]
"""

import collections
import itertools
import sys

from forms import ingredients
from gen1 import zone
from lib import D, R64, exp_v6, load_pdf, load_probe, mpf

ST = {"d": D, "e": R64}


def chains(terms, names):
    """All left-to-right sums over permutations plus ((a+b)+(c+d)) pairings, with storage patterns."""
    n = len(terms)
    for perm in itertools.permutations(range(n)):
        if n >= 3 and perm[0] > perm[1]:
            continue  # first two commute
        for pat in itertools.product("de", repeat=n - 1):
            acc = terms[perm[0]]
            for j, p in zip(perm[1:], pat):
                acc = ST[p](acc + terms[j])
            yield "L2R " + "+".join(names[k] for k in perm) + " " + "".join(pat), acc
    if n == 4:
        for (i, j), (k, l) in [((0, 1), (2, 3)), ((0, 2), (1, 3)), ((0, 3), (1, 2))]:
            for pat in itertools.product("de", repeat=3):
                s1 = ST[pat[0]](terms[i] + terms[j])
                s2 = ST[pat[1]](terms[k] + terms[l])
                yield (
                    f"PAIR ({names[i]}+{names[j]})+({names[k]}+{names[l]}) {''.join(pat)}",
                    ST[pat[2]](s1 + s2),
                )


def main():
    sh = sys.argv[1]
    which = sys.argv[2] if len(sys.argv) > 2 else "S4"
    if sh == "old":
        rows = [r for r in load_pdf() if r["a"] < 1 and 1 <= r["b"] <= 8]
    else:
        rows = [r for r in load_probe("probe1") if r["case"] == sh]
    ing = ingredients(rows)
    z = collections.Counter()
    n = collections.Counter()
    for r, g in zip(rows, ing):
        x = r["x"]
        zn = zone(x)
        n[zn] += 1
        ly = mpf(D(g["l1p"])) if x <= 0.375 else g["ly_yd"]
        tx = mpf(D(g["am1"] * g["LNd"]))
        ty = mpf(D(g["bm1"] * ly))
        ga, gb, gab = g["ga"], g["gb"], g["gab"]
        if which == "S3":
            terms, names = [tx, ty, -mpf(D(ga + D(gb - gab)))], ["tx", "ty", "-L"]
        elif which == "S4":
            terms, names = [tx, ty, -ga, -mpf(D(gb - gab))], ["tx", "ty", "-ga", "-(gb-gab)"]
        else:
            terms, names = [tx, ty, -ga, -gb, gab], ["tx", "ty", "-ga", "-gb", "gab"]
        for name, E in chains(terms, names):
            z[(name, zn)] += exp_v6(E) == r["excel"]
    zones = sorted(n)
    keys = {k[0] for k in z}
    print(sh, which, [(zz, n[zz]) for zz in zones])
    res = sorted(((sum(z[(k, zz)] for zz in zones), k) for k in keys), reverse=True)
    for t, k in res[:10]:
        print(" ", t, k, [z[(k, zz)] for zz in zones])
    for zz in zones:
        b = sorted(((z[(k, zz)], k) for k in keys), reverse=True)[:3]
        print(" best in", zz, n[zz], b)


if __name__ == "__main__":
    main()
