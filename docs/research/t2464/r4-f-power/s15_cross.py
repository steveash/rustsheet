"""s15: binade-crossing scan.  Hypothesis: Excel stores T = fl(K1 + s*V) (one more store) where V is the power term
(V = c*LN(x) extended, or its stored value fl(c*LN(x))), K1 an unknown constant.  For each run, the base residual is
Excel - m1 model with the power product's own rounding removed (est of s13), and for K1 on a fine grid around every
2^k crossing inside the run the residue rho(T) (with sign t = +-1) is fitted with a free offset.  Prints the best
(K1, s, t, V form) per run and its residual sd.  Usage: s15_cross.py SRC MINMAX"""
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P

LDB, S, ex = C.LDB, C.S, C.ex
mpf = mpmath.mpf
U = P.U


def rho_vec(T):
    """T as mpf list -> (fl(T) - T) in units."""
    return np.array([float((mpf(float(t)) - t) * U) for t in T])


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    want = sys.argv[3] if len(sys.argv) > 3 else None
    for blk, rs, m, n in P.runs(src, minmax, want or ""):
        est, Vs = [], {"ext": [], "st": []}
        for r in rs:
            x = r["x"]
            s, e = C.pieces(x, m, n)
            side1 = s["mx"] <= n
            c, sg = (s["a"], 1) if side1 else (s["b"], -1)
            p = LDB(c) * s["Lx"]
            rho = sg * float((ex(S(p)) - ex(p)) * U)
            est.append(P.D_units(r) - float((M.lnF_model(x, m, n, "xl") - mpmath.log(r["exact"])) * U) + rho)
            Vs["ext"].append(ex(p))
            Vs["st"].append(ex(S(p)))
        est = np.array(est)
        best = (est.std(), None)
        for vf, V in Vs.items():
            for sgn in (1, -1):
                for k in range(4, 12):
                    B = mpf(2) ** k
                    # K1 such that K1 + sgn*V crosses B somewhere in the run: K1 = B - sgn*V_j + d
                    v0, v1 = sgn * V[0], sgn * V[-1]
                    lo, hi = B - max(v0, v1), B - min(v0, v1)
                    span = float((hi - lo) * U)
                    nstep = 600
                    for j in range(nstep + 1):
                        K1 = lo + (hi - lo) * j / nstep
                        rr = rho_vec([K1 + sgn * v for v in V])
                        for t in (1, -1):
                            np.column_stack([np.ones(len(est))])
                            res = est - t * rr
                            sd = (res - res.mean()).std()
                            if sd < best[0]:
                                best = (sd, (vf, sgn, k, float(K1), t, span))
        print(f"{blk:20s} est sd {est.std():6.1f} -> {best[0]:6.1f} {best[1]}")
        sys.stdout.flush()


if __name__ == "__main__":
    main()
