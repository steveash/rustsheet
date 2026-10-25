"""Raw-error regression (coefficient = weight in X, units 2^-53) of a block's residual on many candidate roundings of
series-branch quantities, per side; greedy forward selection.  Usage: sparse.py BLOCKREGEX knobs [nsel]"""
import math
import re
import sys

import mpmath
import numpy as np

import mreg as MR
import r3lib as L
import scan as S

LD = L.LDB
U = mpmath.mpf(2) ** 53


def err(exact):
    """Raw rounding error of a stored x87 op on an exact value, units 2^-53."""
    return float((mpmath.mpf(L.DL.D(exact)) - exact) * U)


def feats(x, a, b):
    k, n, nk, ab = L.shape(a, b)
    q = L.st(1 - LD(x))
    f = {}
    for side, kk, M in (("p", k, L.st(LD(n) * LD(x))), ("q", nk, L.st(LD(n) * LD(q)))):
        K, Mm = mpmath.mpf(kk), mpmath.mpf(M)
        d = K - Mm
        sm = mpmath.mpf(L.st(LD(kk) + LD(M)))
        vs = mpmath.mpf(L.st(LD(d) / LD(sm)))
        ve = d / sm
        T = K * mpmath.log(K / Mm)
        for nm, val in (("2kv", 2 * K * vs), ("kv", K * vs), ("2Mv", 2 * Mm * vs), ("Mv", Mm * vs),
                        ("dv", d * vs), ("smv", sm * vs), ("T", T), ("d+dv", d + d * vs),
                        ("2kve", 2 * K * ve), ("dd", d * d), ("bd0", T - d), ("T+M", T + Mm),
                        ("4kv", 4 * K * vs), ("2kv2", 2 * K * vs * vs), ("kLN", K * mpmath.mpf(L.LN(L.st(LD(kk) / LD(M)))))):
            f[side + "." + nm] = err(val)
        f[side + ".dvx"] = float(d * (vs - ve) * U)  # stored-v effect on d*v
    return f


def main(argv):
    src = "old"
    if argv[0] == "--src":
        src, argv = argv[1], argv[2:]
    rows = S.load(src)
    pat = re.compile(argv[0])
    o = S.parse(argv[1])
    nsel = int(argv[2]) if len(argv) > 2 else 6
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk) or len(idx) < 64:
            continue
        r = MR.resid(rows, idx, o)
        F = [feats(rows[i]["x"], rows[i]["a"], rows[i]["b"]) for i in idx]
        names = list(F[0])
        A = np.array([[z[n] for n in names] for z in F])
        sel = []
        print(f"{blk}: sd {np.std(r):.2f}")
        for _ in range(nsel):
            best = None
            for j, n in enumerate(names):
                if j in sel or np.std(A[:, j]) < 1e-9:
                    continue
                cols = sel + [j]
                M_ = np.c_[A[:, cols], np.ones(len(r))]
                co, *_ = np.linalg.lstsq(M_, r, rcond=None)
                sd = np.std(r - M_ @ co)
                if best is None or sd < best[0]:
                    best = (sd, j, co)
            sel.append(best[1])
            print(f"   + {names[best[1]]:8s} sd {best[0]:.2f}  coefs " +
                  " ".join(f"{names[c]}={v:+.2f}" for c, v in zip(sel, best[2])))


if __name__ == "__main__":
    main(sys.argv[1:])
