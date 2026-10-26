"""m9: K as an arbitrary value K' + delta (delta in units, extended), structure fl(fl(K + H) + P) - ln x (HKP11 of
m7, value-space x^-1) or a given m7 order/flags.  Per run: best delta by residual sd, and the sd profile.
Usage: m9_kscan.py SRC MINMAX TAG [ORDER FLAGS] [STEP] [RANGE]"""
import sys

import mpmath
import numpy as np

import cands as C
import m7_flex as M7
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U
mpf = mpmath.mpf


def run_scan(rs, m, n, order, flags, step, rng):
    D = np.array([P.D_units(r) for r in rs])
    exl = [mpmath.log(r["exact"]) for r in rs]
    side1 = m * rs[0]["x"] <= n  # runs never straddle the side switch here
    K0 = s3_order.kprime(m, n, side1)
    T = [M7.terms(r["x"], m, n, 0, 0) for r in rs]
    out = []
    for dl in np.arange(-rng, rng + step / 2, step):
        Kd = P.F.R3._ld_of(K0 + mpf(float(dl)) / U)
        Dm = []
        for (t, lx), e in zip(T, exl):
            t = dict(t, K=Kd)
            Dm.append(float((M7.value(t, lx, order, flags) - e) * U))
        d = D - np.array(Dm)
        out.append((d.std(), float(dl), float(d.mean())))
    return out


def main():
    src, minmax, tag = sys.argv[1], float(sys.argv[2]), sys.argv[3]
    order = sys.argv[4] if len(sys.argv) > 4 else "HKP"
    flags = tuple(int(c) for c in (sys.argv[5] if len(sys.argv) > 5 else "11"))
    step = float(sys.argv[6]) if len(sys.argv) > 6 else 2.0
    rng = float(sys.argv[7]) if len(sys.argv) > 7 else 256.0
    for blk, rs, m, n in P.runs(src, minmax, tag):
        out = run_scan(rs, m, n, order, flags, step, rng)
        best = sorted(out)[:3]
        print(f"{blk:20s} {order}{''.join(map(str, flags))} " + " | ".join(f"delta {b[1]:+.0f} sd {b[0]:.1f} mean {b[2]:+.0f}" for b in best))
        sys.stdout.flush()


if __name__ == "__main__":
    main()
