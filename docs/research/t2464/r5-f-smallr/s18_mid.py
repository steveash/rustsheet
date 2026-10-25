"""A second constant between H and P: chain CHAIN (default 'C+H!+D!+P!'), C + D = the run's constant.  D is set to
+-1.5*2^k (k = 3..10) and to natural pieces, its phase scanned over one ulp of S1 + D, C over +-ulp(S1).
Usage: s18_mid.py SET tag [chain]"""
import sys

import mpmath
import numpy as np

import r5lib as CL
import r5lib as R

LD = R.LD


def best_for_D(rs, m, n, chain, D0):
    toks, arr, meas, K0 = CL.prepare(rs, m, n, chain)
    H0 = float(arr["H"][0])
    S1 = float(K0) - float(D0) + H0
    g1 = R.ulpu(S1)
    g2 = R.ulpu(S1 + float(D0))
    return CL.scan2(rs, m, n, chain, mpmath.mpf(float(D0)), spanC=max(g1, 4.0), spanD=max(g2 / 2, 2.0),
                    stepC=0.25, stepD=max(0.25, g2 / 256))[0]


if __name__ == "__main__":
    src, tag = sys.argv[1], sys.argv[2]
    chain = sys.argv[3] if len(sys.argv) > 3 else "C+H!+D!+P!"
    pred = R.small_r if tag == "all" else (lambda m, n, rs: tag in rs[0]["blk"])
    for blk, rs, m, n in R.runs(src, pred):
        line = []
        for sg in (1, -1):
            for k in range(3, 11):
                D = sg * 1.5 * 2.0 ** k
                line.append((best_for_D(rs, m, n, chain, D), D))
        best = max(line)
        print(f"{blk:28s} best {best[0]} at D~{best[1]:g} |", " ".join(f"{D:g}:{s}" for s, D in line))
        sys.stdout.flush()
