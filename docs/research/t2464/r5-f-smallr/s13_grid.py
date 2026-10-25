"""Which grid does the first stored sum T = fl(D + H) have?  Chain 'D+H!+C+P!' (T stored, then C + P added with one
final rounding), D placed so that T lies in a chosen binade [2^k, 2^(k+1)) (sign +/-), its phase scanned over one
ulp of T, C scanned over +-g2.  Prints per run the score per (sign, k).  Usage: s13_grid.py SET tag [chain]"""
import sys

import mpmath
import numpy as np

import r5lib as CL
import r5lib as R

LD = R.LD


def score_bin(rs, m, n, chain, Tmag, gfinal):
    toks, arr, meas, K0 = CL.prepare(rs, m, n, chain)
    H0 = mpmath.mpf(float(arr["H"][0]))
    D0 = Tmag - H0
    g1 = R.ulpu(float(Tmag))
    return CL.scan2(rs, m, n, chain, D0, spanC=max(gfinal, 8.0), spanD=max(g1 / 2, 1.0), stepC=0.25,
                    stepD=max(0.25, g1 / 256))[0]


if __name__ == "__main__":
    src, tag = sys.argv[1], sys.argv[2]
    chain = sys.argv[3] if len(sys.argv) > 3 else "D+H!+C+P!"
    pred = R.small_r if tag == "all" else (lambda m, n, rs: tag in rs[0]["blk"])
    for blk, rs, m, n in R.runs(src, pred):
        side1 = R.base(rs[0]["x"], m, n)[0]
        A, B = CL.loader_split(m, n, side1)
        toks, arr, meas, K0 = CL.prepare(rs, m, n, "C+H!+P!")
        S2 = float(meas[0])
        g2 = R.ulpu(S2)
        H0 = float(arr["H"][0])
        line = []
        for sg in (1, -1):
            for k in range(-2, 10):
                T = sg * 1.5 * 2.0 ** k
                line.append((score_bin(rs, m, n, chain, mpmath.mpf(T), g2), T))
        best = max(line)
        print(f"{blk:28s} H={H0:8.3f} K'+H={float(K0) + H0:8.3f} A+H={float(A) + H0:8.3f} B+H={float(B) + H0:8.3f} "
              f"| best {best[0]} at T~{best[1]:g} |", " ".join(f"{T:g}:{s}" for s, T in line))
        sys.stdout.flush()
