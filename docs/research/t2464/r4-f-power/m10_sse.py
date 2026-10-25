"""m10: fl(fl(K + H) + P) - ln x with every rounding either x87 (fl53(fl64(.))) or SSE (one correct rounding),
K any double (or extended) near K' + delta, H = -h*L(r) extended or stored, P = sg*fl(c*LN(x)) (its product rounded
the same way).  Per run, the best delta (units, step 1) by residual sd and the count of cells off the modal residual.
Usage: m10_sse.py SRC MINMAX TAG RANGE"""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

LDB = C.LDB
U = P.U
mpf = mpmath.mpf
fl53, fl64 = P.F.L.R64, None


def rnd(v, mode):
    """Round an mpf to a double: x87 = via 64 bits, sse = once."""
    import excelmath as XM

    return mpf(XM.fl53(XM.fl64(v))) if mode == "x87" else mpf(XM.fl53(v))


def prep(rs, m, n, mode, hst):
    out = []
    for r in rs:
        s, _ = C.pieces(r["x"], m, n)
        a, b, h = s["a"], s["b"], s["h"]
        side1 = s["mx"] <= n
        c, sg = (a, 1) if side1 else (b, -1)
        Lx = C.ex(s["Lx"])
        Pv = sg * rnd(mpf(c) * Lx, mode)
        Lr = C.ex(M.L_form(s, side1, "xl", n))
        H = -mpf(h) * Lr
        if hst:
            H = rnd(H, mode)
        out.append((Pv, H, mpmath.log(mpf(r["x"])), mpmath.log(r["exact"])))
    return out


def score(pre, D, K, mode):
    Dm = np.array([float((rnd(rnd(K + H, mode) + Pv, mode) - lx - e) * U) for Pv, H, lx, e in pre])
    d = D - Dm
    vals, cnt = np.unique(np.round(d), return_counts=True)
    mode_v = vals[np.argmax(cnt)]
    off = int(np.sum(np.abs(d - mode_v) > 3))
    return d.std(), off, float(np.median(d))


def main():
    src, minmax, tag, rng = sys.argv[1], float(sys.argv[2]), sys.argv[3], int(sys.argv[4])
    for blk, rs, m, n in P.runs(src, minmax, tag):
        D = np.array([P.D_units(r) for r in rs])
        K0 = s3_order.kprime(m, n, m * rs[0]["x"] <= n)
        line = []
        for mode in ("x87", "sse"):
            for hst in (0, 1):
                pre = prep(rs, m, n, mode, hst)
                best = None
                for dl in range(-rng, rng + 1):
                    K = K0 + mpf(dl) / U
                    sd, off, med = score(pre, D, K, mode)
                    if best is None or (off, sd) < (best[1], best[0]):
                        best = (sd, off, med, dl)
                line.append(f"{mode} Hs={hst}: d={best[3]:+d} off {best[1]} sd {best[0]:.1f}")
        print(f"{blk:20s} n={len(rs)} | " + " | ".join(line))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
