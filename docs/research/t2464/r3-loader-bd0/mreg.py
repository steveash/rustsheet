"""Multiple regression of a block's residual r = D - dX on chosen mfilter features (rounding errors in ulps).
Usage: mreg.py [--src S] BLOCKREGEX 'knobs' feat1 feat2 ..."""
import re
import sys

import mpmath
import numpy as np

import mfilter as MF
import r3lib as L
import rmsfit as RF
import scan as S


def resid(rows, idx, o):
    Ds = RF.block_D(rows, idx)
    res = []
    for i, (d, xe) in zip(idx, Ds):
        b1, b2, _ = L.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], o)
        xm = L.frac(-b1 - b2)
        res.append(d - float((mpmath.mpf(xm.numerator) / xm.denominator - xe) * RF.U))
    return np.array(res)


def main(argv):
    src = "old"
    if argv[0] == "--src":
        src, argv = argv[1], argv[2:]
    rows = S.load(src)
    pat = re.compile(argv[0])
    o = S.parse(argv[1])
    names = argv[2:]
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk) or len(idx) < 64:
            continue
        r = resid(rows, idx, o)
        F = np.array([[MF.cands(rows[i]["x"], rows[i]["a"], rows[i]["b"])[n] for n in names] for i in idx])
        A = np.c_[F, np.ones(len(r))]
        co, *_ = np.linalg.lstsq(A, r, rcond=None)
        print(f"{blk:12s} sd {np.std(r):6.2f} -> {np.std(r - A @ co):6.2f}  " +
              " ".join(f"{n}={c:+.2f}" for n, c in zip(names, co)))


if __name__ == "__main__":
    main(sys.argv[1:])
