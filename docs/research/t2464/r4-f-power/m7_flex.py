"""m7: flexible double-op sum.  Terms: K (double nearest the exact K', moved j ulps), H = -h*L(r) (extended, or
stored with 'Hs'), P = sg*fl(c*LN(x)); the x^-1 factor as exact -ln x (value-space division) or -LN(x) ('X' in the
order).  The order lists the summands; flags give, per partial sum, stored (1) or kept extended (0).
Per run: the best (order, flags, j, H storage) by residual sd.  Usage: m7_flex.py SRC MINMAX TAG J"""
import itertools
import math
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U
mpf = mpmath.mpf


def terms(x, m, n, j, hst):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    k = float(s3_order.kprime(m, n, side1))
    for _ in range(abs(j)):
        k = math.nextafter(k, math.inf if j > 0 else -math.inf)
    H = -LDB(h) * M.L_form(s, side1, "xl", n)
    if hst:
        H = S(H)
    return {"K": LDB(k), "P": sg * S(LDB(c) * s["Lx"]), "H": H, "X": -s["Lx"]}, mpmath.log(mpf(x))


def value(t, lnx, order, flags):
    acc = t[order[0]]
    for i, k in enumerate(order[1:]):
        acc = acc + t[k]
        if flags[i]:
            acc = S(acc)
    v = ex(acc)
    if "X" not in order:
        v = v - lnx
    return v


def candidates():
    for order in itertools.chain(itertools.permutations("KHP"), itertools.permutations("KHPX")):
        for flags in itertools.product((0, 1), repeat=len(order) - 1):
            yield "".join(order), flags


def main():
    src, minmax, tag, J = sys.argv[1], float(sys.argv[2]), sys.argv[3], int(sys.argv[4])
    for blk, rs, m, n in P.runs(src, minmax, tag):
        D = np.array([P.D_units(r) for r in rs])
        exl = [mpmath.log(r["exact"]) for r in rs]
        out = []
        for j in range(-J, J + 1):
            for hst in (0, 1):
                T = [terms(r["x"], m, n, j, hst) for r in rs]
                for order, flags in candidates():
                    Dm = np.array([float((value(t, lx, order, flags) - e) * U) for (t, lx), e in zip(T, exl)])
                    d = D - Dm
                    out.append((d.std(), order, flags, j, hst, float(d.mean())))
        out.sort()
        print(f"{blk:20s} " + " | ".join(f"{o[1]}{''.join(map(str, o[2]))} j={o[3]:+d} Hs={o[4]} sd {o[0]:.1f} m {o[5]:+.0f}" for o in out[:4]))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
