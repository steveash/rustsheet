"""a6: within-shape consistency of composition structures with free shape constants (E extended into EXP).

  ext  : E = C + X                                   (one free C)
  g    : E = LN(n+1) + fl53(X + c)                   (lc - lf/2 stored, then + L5 in extended; one free c)
  g2   : E = c1 + fl53(X + c2)                       (a stored partial sum; c2 scanned on a 1/16-ulp grid, c1 free)
  lc   : E = c1 + fl53(X + Cs)  (lc stored; Cs scanned +-5 units around the exact stirlerr sum, c1 free)
  vs   : y = fl53(EXP_V6(C + X) * f), f = fl(a+b) - 1  (R 2.x dbeta: p * f in value space; C = lc - lf/2 part)
Per shape the largest set of cells sharing one constant is found (sweep over interval endpoints); prints the
consistent share. A positive control (synthetic Excel values from the structure itself) runs with --control.
Usage: a6_struct.py sweep|diverse STRUCT [--control] [--max-shapes N] [--t V0|V1|V2|V3|V4] [--base a,b] [--min-k K] [--bd0st]
  --t: X from r3lib.xpart_t with that log-form t variant (default: r2's xpart_v 'R'); --m its M form.
"""

from __future__ import annotations

import math
import random
import sys
from collections import defaultdict

import mpmath

import a1_pin as A1
import a3_const as A3
import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
D = R.L.D


def g_cands(lo, hi):
    """Doubles g in [lo, hi] (real interval), at most a few."""
    out = []
    g = float(lo)
    g = math.nextafter(g, -math.inf)
    while g <= hi:
        if g >= lo:
            out.append(g)
        g = math.nextafter(g, math.inf)
        if len(out) > 64:
            break
    return out


def c_set_for_stored(eint, base, X):
    """Intervals of c with base + fl53(X + c) in eint (real interval of E)."""
    out = []
    for g in g_cands(eint[0] - base, eint[1] - base):
        h1 = (g - math.nextafter(g, -math.inf)) / 2
        h2 = (math.nextafter(g, math.inf) - g) / 2
        out.append((mpf(g) - h1 - X, mpf(g) + h2 - X))
    return out


def vs_set(y, f, X):
    """Intervals of C with fl53(EXP_V6(C + X) * f) == y."""
    out = []
    w0 = float(mpf(y) / f)
    for j in range(-3, 4):
        w = R.MP.step(w0, j) if j else w0
        if float(D(mpf(w) * f)) != y:
            continue
        ei = R.e_interval_ext(w)
        if ei is not None:
            out.append((ei[0] - X, ei[1] - X))
    return out


def best_overlap_sets(sets):
    ivs = []
    for cid, s in enumerate(sets):
        ivs += [(lo, hi) for lo, hi in s]
    return A1.best_overlap(ivs) if ivs else (0, None, None)


def main():
    which, struct = sys.argv[1], sys.argv[2]
    ctl = "--control" in sys.argv
    rows = R.load(which)
    by = defaultdict(list)
    for r in rows:
        by[(r["a"], r["b"])].append(r)
    shapes = sorted(by)
    if "--base" in sys.argv:  # only the ulp-neighbourhood of one sweep base
        a0, b0 = (float(v) for v in sys.argv[sys.argv.index("--base") + 1].split(","))
        shapes = [s for s in shapes if abs(s[0] - a0) < 1e-9 * a0 and abs(s[1] - b0) < 1e-9 * b0]
    if "--min-k" in sys.argv:  # only shapes with min(k, n-k) >= this
        mk = float(sys.argv[sys.argv.index("--min-k") + 1])
        shapes = [s for s in shapes if min(s[0], s[1]) - 1 >= mk]
    if "--max-shapes" in sys.argv:
        random.Random(7).shuffle(shapes)
        shapes = shapes[: int(sys.argv[sys.argv.index("--max-shapes") + 1])]
    tm = sys.argv[sys.argv.index("--t") + 1] if "--t" in sys.argv else None
    mm = sys.argv[sys.argv.index("--m") + 1] if "--m" in sys.argv else "nx"
    tot = cons = 0
    rng = random.Random(11)
    for sh in shapes:
        rs = by[sh]
        k, n, nk = (mpf(v) for v in R.shape_pieces(*sh))
        L5 = mpf(R.L.LN(float(D(n + 1))))
        cex = R.exact_const(*sh)
        c2s = [None]
        if struct == "g2":
            c2s = [cex - L5 + j / 16 * U * 2 for j in range(-16, 17)]
        if struct == "lc":  # lc = fl53(Cs + X) stored: c2 = Cs scanned around the exact stirlerr sum
            cs = A3.stirlerr(n, ("exact", "exact")) - A3.stirlerr(k, ("exact", "exact")) - A3.stirlerr(nk, ("exact", "exact"))
            c2s = [cs + j / 8 * U for j in range(-40, 41)]
        best = 0
        # synthetic control: true constants near the exact one with a random fractional offset
        tc = cex - L5 + rng.uniform(-2, 2) * U
        for c2 in c2s:
            sets = []
            for r in rs:
                X = R.xpart_t(r["x"], *sh, tm, mm, "--bd0st" in sys.argv) if tm else R.xpart_v(r["x"], *sh, "R")
                y = r["excel"]
                f = mpf(D(n + 1))
                if ctl:
                    E = cex + 1.3 * U + X if struct == "ext" else L5 + mpf(D(X + tc))
                    y = R.L.exp_v6(E)
                    if struct == "vs":
                        y = float(D(mpf(R.L.exp_v6(cex - mpmath.log(f) + 1.3 * U + X)) * f))
                if struct == "vs":
                    sets.append(vs_set(y, f, X))
                    continue
                ei = R.e_interval_ext(y)
                if ei is None:
                    sets.append([])
                    continue
                if struct == "ext":
                    sets.append([(ei[0] - X, ei[1] - X)])
                elif struct == "g":
                    sets.append(c_set_for_stored(ei, L5, X))
                else:  # g2 / lc: c1 + fl53(X + c2) in ei  ->  c1 interval
                    g = mpf(D(X + c2))
                    sets.append([(ei[0] - g, ei[1] - g)])
            nn, lo, hi = best_overlap_sets(sets)
            best = max(best, nn)
        tot += len(rs)
        cons += best
    R.save_cache()
    print(f"{which} {struct}{' control' if ctl else ''}{' t=' + tm + ' m=' + mm if tm else ''}{' bd0st' if '--bd0st' in sys.argv else ''}: {cons}/{tot} cells consistent ({cons / tot:.3f}) over {len(shapes)} shapes")


if __name__ == "__main__":
    main()
