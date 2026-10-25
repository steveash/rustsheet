"""s7: Loader-style piece sums with NO free constant.  ln F = ST + A + B + P + H (- ln x), with
  ST = stirlerr(h) - stirlerr(a) - stirlerr(b), each stirlerr the nearest double of the exact value (R's table for
       half-integers <= 15); 'T' = ST as one stored double, or the three pieces as separate summands (s1, s2, s3)
  A  = (h - 1/2) * L(fl(w/o)) (Excel's log1p; product stored 'As' or extended 'Ae')
  B  = 1/2 LN(w) - ln sqrt(2 pi) (fl of each piece; at w = 1 it is -M_LN_SQRT_2PI)
  P  = sg*fl(c LN x), H = -h L(r) (extended), X = -LN(x) (when present the final step is plain EXP, else /x)
Every ordering of the summands with every store/extend flag pattern is scored by bit-exact count against Excel.
Usage: s7_pieces.py SET MAXD [--x] [--split] [--shapes m,n;m,n]"""

import itertools
import sys

import mpmath
import numpy as np

import clib as C

LDB, st = C.LDB, C.st
F = C.F
LN_SQRT_2PI = 0.918938533204672741780329736406


def stirl(v: float) -> LDB:
    return LDB(float(F.stirlerr_exact(v)))


def terms(r, ast: bool, split: bool):
    x, m, n = r["x"], r["d1"], r["d2"]
    pc = C.pieces(x, m, n)
    a, b, h = pc["a"], pc["b"], pc["h"]
    w, o = (a, b) if pc["side"] == 1 else (b, a)
    A = LDB(h - 0.5) * C.SL.L_xl(st(LDB(w) / LDB(o)))
    if ast:
        A = LDB(st(A))
    B = LDB(st(LDB(0.5) * C.SL.LN(w))) - LDB(LN_SQRT_2PI) if w != 1 else LDB(-LN_SQRT_2PI)
    B = LDB(st(B))
    t = {"A": A, "B": B, "P": LDB(pc["P"]), "H": pc["H"], "X": -LDB(C.SL.LN(x))}
    if split:
        t.update({"1": stirl(h), "2": -stirl(a), "3": -stirl(b)})
    else:
        t["T"] = LDB(st(stirl(h) - stirl(a) - stirl(b)))
    return t


def chain(order, flags, t):
    acc = t[order[0]]
    for i, k in enumerate(order[1:]):
        acc = acc + t[k]
        if flags[i]:
            acc = LDB(st(acc))
    return acc


def main():
    a = sys.argv[1:]
    name, maxd = a[0], float(a[1])
    withx, split = "--x" in a, "--split" in a
    shapes = (
        {tuple(float(v) for v in s.split(",")) for s in a[a.index("--shapes") + 1].split(";")}
        if "--shapes" in a
        else None
    )
    rows = [
        r
        for r in C.load(name)
        if max(r["d1"], r["d2"]) <= maxd and (not shapes or (r["d1"], r["d2"]) in shapes)
    ]
    rows = [r for r in rows if C.SL.side_r(r["x"], r["d1"], r["d2"])[1] > 0.375]
    names = ("1", "2", "3") if split else ("T",)
    base = names + ("A", "B", "P", "H") + (("X",) if withx else ())
    res = []
    for ast in (False, True):
        T = [terms(r, ast, split) for r in rows]
        X = [LDB(r["x"]) for r in rows]
        for order in itertools.permutations(base):
            for flags in itertools.product((0, 1), repeat=len(order) - 1):
                c = 0
                for t, xx, r in zip(T, X, rows):
                    E = chain(order, flags, t)
                    v = C.R3.exp_v6(E) if withx else st(C.R3.exp_v6(E, True) / xx)
                    c += v == r["excel"]
                res.append((c, "".join(order), "".join(map(str, flags)), ast))
    res.sort(reverse=True)
    cr = sum(r["cr"] == r["excel"] for r in rows)
    print(f"{len(rows)} cells, CR {cr}")
    for c, o, f, ast in res[:15]:
        print(f"{c} {o} {f} {'As' if ast else 'Ae'}")


if __name__ == "__main__":
    main()
