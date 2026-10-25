"""Exhaustive assembly of bpser's EXP argument z = alx - lnB from its stored pieces, on the unswapped w_bpser cells:
I = fl(EXP(z) * s), s = w3's series (fl(1/a) forward, x87 stored), alx = fl(a LN x). lnB's pieces for a0 = min < 1:
  b0 < 8:  g(a0), G(b0), GL(a0 + b0)        (+ g, G: "g1ln53" fl(gamln1(v) - LN v), "g1ln64" extended, "gl" GAMMALN;
                                             G(b0) = GAMMALN(b0) when b0 >= 1)
  b0 >= 8: g(a0), algdiv(a0, b0)            (TOMS algdiv, Excel LN; TOMS's own u = gamln1(a0) + algdiv also tried:
                                             lnB = u - LN(a0) with u's pieces as leaves)
Every full binary tree over the signed leaves (alx, -g, -G, +GL), every rounding of each internal node (fl53 stored /
fl64 register), root rounded 53 (EXP of a double) or 64 (EXP of an extended argument). Counts exact matches per
form on tune / hold-out (shape-level md5 half) / per set.
usage: zforms.py sub=<1<b0<8|b0<=1|b0>=8> [sets=runs,shapes] [top=25] [plant=g1ln53|gl|g1ln64 (positive control)]"""

from __future__ import annotations

import itertools
import sys
from collections import Counter, defaultdict

import mpmath

import lib
from lib import GAMMALN, LN, mpf, sx

mpmath.mp.prec = 200
K = lib.Model(lib.bx.BEST, lib.V())


def gterm(form, v):
    if v >= 1.0:
        return mpf(GAMMALN(v))
    if form == "gl":
        return mpf(GAMMALN(v))
    g = mpf(K.gamln1(v)) - mpf(LN(v))
    return lib.fl64(g) if form == "g1ln64" else mpf(lib.fl53(g))


def trees(leaves):
    """All full binary trees over the leaf indices (unordered children): nested tuples."""
    if len(leaves) == 1:
        yield leaves[0]
        return
    first, rest = leaves[0], leaves[1:]
    # split the set into two non-empty parts, the part with `first` on the left (avoids mirrored duplicates)
    for r in range(0, len(rest)):
        for comb in itertools.combinations(rest, r):
            left = (first,) + comb
            right = tuple(i for i in rest if i not in comb)
            for lt in trees(left):
                for rt in trees(right):
                    yield (lt, rt)


def n_internal(t):
    return 0 if not isinstance(t, tuple) else 1 + n_internal(t[0]) + n_internal(t[1])


def evaluate(t, vals, rnd, pos=None):
    """Evaluate tree t: leaves index vals (signed mpf); internal nodes rounded by rnd[k] in preorder."""
    if pos is None:
        pos = [0]
    if not isinstance(t, tuple):
        return vals[t]
    k = pos[0]
    pos[0] += 1
    left = evaluate(t[0], vals, rnd, pos)
    right = evaluate(t[1], vals, rnd, pos)
    v = left + right
    return lib.fl64(v) if rnd[k] == 64 else mpf(lib.fl53(v))


def show(t, names):
    if not isinstance(t, tuple):
        return names[t]
    return f"({show(t[0], names)} {show(t[1], names)})"


def main():
    kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
    sub = "bpser:" + kw["sub"]
    sets = tuple(kw.get("sets", "runs,shapes").split(","))
    rows = lib.load_probe(sets)
    res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], lib.V())
    cells = []
    for r, q in zip(rows, res):
        if q[2] != "w_bpser" or not q[3]:
            continue
        fn, a, b, x = q[3][-1]
        if lib.sub_branch(fn, a, b) != sub:
            continue
        cells.append(
            (a, b, x, r["excel"], lib.Model.series_w3(a, b, x), mpf(sx("*", a, LN(x))), r["set"], r["hold"])
        )
    if "plant" in kw:  # positive control: replace Excel by the planted form (tree (alx (-g (-G +GL))), (64, 53, 53))
        gf0 = kw["plant"]
        planted = []
        for a, b, x, e, s, alx, st, hold in cells:
            lo, hi = min(a, b), max(a, b)
            gh = gterm(gf0, hi) if hi < 1.0 else mpf(GAMMALN(hi))
            L = lib.fl53(gterm(gf0, lo) + mpf(lib.fl53(gh - mpf(GAMMALN(sx("+", lo, hi))))))
            planted.append((a, b, x, sx("*", lib.EXPX(lib.fl64(alx - mpf(L))), s), s, alx, st, hold))
        cells = planted
        print("POSITIVE CONTROL: planted g =", gf0)
    print(sub, len(cells), "cells", len({(c[0], c[1]) for c in cells}), "shapes")
    big = sub == "bpser:b0>=8"
    gforms = ("g1ln53", "g1ln64", "gl")
    hforms = gforms if sub == "bpser:b0<=1" else ("-",)
    score = defaultdict(Counter)
    for gf, hf in itertools.product(gforms, hforms):
        leafsets = []
        for a, b, x, e, s, alx, st, hold in cells:
            lo, hi = min(a, b), max(a, b)
            if big:
                ad = mpf(K.algdiv(lo, hi))
                leafsets.append([alx, -gterm(gf, lo), -ad])
            else:
                gh = gterm(hf, hi) if hi < 1.0 else mpf(GAMMALN(hi))
                leafsets.append([alx, -gterm(gf, lo), -gh, mpf(GAMMALN(sx("+", lo, hi)))])
        names = ["alx", "-g", "-algdiv"] if big else ["alx", "-g", "-G", "+GL"]
        for t in trees(tuple(range(len(names)))):
            ni = n_internal(t)
            for rnd in itertools.product((53, 64), repeat=ni):
                key = (gf, hf, show(t, names), rnd)
                sc = score[key]
                for (a, b, x, e, s, alx, st, hold), vals in zip(cells, leafsets):
                    z = evaluate(t, vals, rnd)
                    ev = lib.EXP(float(z)) if rnd[0] == 53 else lib.EXPX(z)
                    ok = sx("*", ev, s) == e
                    sc["all"] += ok
                    sc["hold" if hold else "tune"] += ok
                    sc[st] += ok
    nt = Counter(("hold" if c[7] else "tune") for c in cells)
    ns = Counter(c[6] for c in cells)
    print(f"cells: tune {nt['tune']}, hold-out {nt['hold']}, " + ", ".join(f"{k} {v}" for k, v in ns.items()))
    best = sorted(score.items(), key=lambda kv: -kv[1]["tune"])[: int(kw.get("top", "25"))]
    for key, sc in best:
        print(
            f"tune {sc['tune']:5d} hold {sc['hold']:5d} | "
            + " ".join(f"{k} {sc[k]}" for k in ns)
            + f" | g={key[0]} G={key[1]} tree={key[2]} round(preorder)={key[3]}"
        )


if __name__ == "__main__":
    main()
