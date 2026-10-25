"""m8: the constant split into pieces summed with the x terms, every partial sum a stored double op.
Pieces: G = +lnGamma(h), A = -lnGamma(a), B = -lnGamma(b) (nearest doubles of the exact values), R = the side's
ratio constant (+a ln(m/n) | -b ln(m/n)) as the nearest double ('Rn') or the stored product fl(c*LN(fl(m/n)))
('Rp'), P = sg*fl(c*LN(x)), H = -h*L(r); x^-1 by value-space division (exact -ln x) or as a summand X = -LN(x).
All orders of the summands; last sum extended ('e') or stored ('x').  Per run, the best candidates.
Usage: m8_pieces.py SRC MINMAX TAG [Rmode] [withX]"""
import collections
import itertools
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U
mpf = mpmath.mpf


def pieces(x, m, n, rmode):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    A_, B_, H_ = mpf(a), mpf(b), mpf(h)
    lr = mpmath.log(mpf(m) / mpf(n))
    if rmode == "Rn":
        R = LDB(float((A_ if side1 else -B_) * lr))
    else:
        cr = a if side1 else -b
        R = S(LDB(cr) * C.LNx(C.st(LDB(m) / LDB(n))))
    t = {"G": LDB(float(mpmath.loggamma(H_))), "A": LDB(-float(mpmath.loggamma(A_))),
         "B": LDB(-float(mpmath.loggamma(B_))), "R": R, "P": sg * S(LDB(c) * s["Lx"]),
         "H": -LDB(h) * M.L_form(s, side1, "xl", n), "X": -s["Lx"]}
    if m == n:
        t["R"] = LDB(0)
    return t, mpmath.log(mpf(x))


def value(t, lnx, order, last):
    acc = t[order[0]]
    for i, k in enumerate(order[1:]):
        acc = acc + t[k]
        if i < len(order) - 2 or last == "x":
            acc = S(acc)
    v = ex(acc)
    if "X" not in order:
        v = v - lnx
    return v


def main():
    src, minmax, tag = sys.argv[1], float(sys.argv[2]), sys.argv[3]
    rmode = sys.argv[4] if len(sys.argv) > 4 else "Rn"
    withx = len(sys.argv) > 5 and sys.argv[5] == "X"
    keys = "GABRPH" + ("X" if withx else "")
    agg = collections.defaultdict(list)
    for blk, rs, m, n in P.runs(src, minmax, tag):
        D = np.array([P.D_units(r) for r in rs])
        exl = [mpmath.log(r["exact"]) for r in rs]
        T = [pieces(r["x"], m, n, rmode) for r in rs]
        out = []
        for order in itertools.permutations(keys):
            if m == n and order.index("R") != len(order) - 1:
                continue  # R = 0 at m = n: its position is immaterial; keep one
            for last in ("e", "x"):
                Dm = np.array([float((value(t, lx, order, last) - e) * U) for (t, lx), e in zip(T, exl)])
                d = D - Dm
                out.append((d.std(), "".join(order) + last, float(d.mean())))
        out.sort()
        for sd, k, _ in out:
            agg[k.replace("R", "") if m == n else k].append(sd)
        print(f"{blk:20s} " + " | ".join(f"{o[1]} sd {o[0]:.1f} m {o[2]:+.0f}" for o in out[:5]))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
