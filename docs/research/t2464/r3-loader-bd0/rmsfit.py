"""Chain-free test of an x-part model: per block, D = Excel's log error (ln(excel) - exact ln pdf, units 2^-53) against
dX = model X - exact X (exact Loader with exact np, nq, q).  Prints sd(D), sd(D - dX - mean) and the regression slope
of D on dX (1 for the right model).  Usage: rmsfit.py [--src S] BLOCKREGEX 'knobs' ...  """
import math
import re
import sys

import mpmath
import numpy as np

import r3lib as L
import scan as S

U = mpmath.mpf(2) ** 53


def exact_X(x, a, b):
    """-bd0(k, n x) - bd0(nk, n(1-x)) exactly, with exact k, n (a-1, a+b-2) and exact 1-x."""
    k, n = mpmath.mpf(a) - 1, mpmath.mpf(a) + mpmath.mpf(b) - 2
    nk = n - k
    X = mpmath.mpf(x)

    def bd(kk, M):
        return kk * mpmath.log(kk / M) + M - kk

    return -bd(k, n * X) - bd(nk, n * (1 - X))


_EX: dict = {}


def block_D(rows, idx):
    out = []
    for i in idx:
        r = rows[i]
        key = (r["x"], r["a"], r["b"])
        if key not in _EX:
            _EX[key] = (L.exact_logpdf(*key), exact_X(*key))
        lp, xe = _EX[key]
        out.append((float((mpmath.log(r["excel"]) - lp) * U), xe))
    return out


def run(rows, idx, o):
    Ds = block_D(rows, idx)
    D = np.array([d for d, _ in Ds])
    dX = np.array([float((mpmath.mpf(L.frac(-b1 - b2).numerator) / L.frac(-b1 - b2).denominator - xe) * U)
                   for (d, xe), (b1, b2, _) in zip(Ds, (L.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], o)
                                                         for i in idx))])
    r = D - dX
    A = np.c_[dX - dX.mean(), np.ones(len(D))]
    sl = np.linalg.lstsq(A, D, rcond=None)[0][0]
    fl = []
    for i in idx:  # chain floor: uniform roundings of E and of lc = E - L5 + h (units), plus the output's
        E = math.log(rows[i]["excel"])
        L5, h = L.shape_pieces(rows[i]["a"], rows[i]["b"])
        lc = E - L5 + h
        uo = math.ulp(rows[i]["excel"]) / rows[i]["excel"] * 2**53
        fl.append((math.ulp(E) ** 2 + math.ulp(lc) ** 2) * 2**106 / 12 + uo**2 / 12)
    return np.std(D), np.std(r), sl, np.std(dX), math.sqrt(np.mean(fl))


def main(argv):
    src = "old"
    if argv[0] == "--src":
        src, argv = argv[1], argv[2:]
    rows = S.load(src)
    pat = re.compile(argv[0])
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk) or len(idx) < 64:
            continue
        r0 = rows[idx[0]]
        for spec in argv[1:] or [""]:
            sD, sr, sl, sx, fl = run(rows, idx, S.parse(spec))
            print(f"{blk:16s} ({r0['a']:g},{r0['b']:g}) [{spec:24s}] sd(D) {sD:6.2f} sd(dX) {sx:6.2f} "
                  f"sd(D-dX) {sr:6.2f} floor {fl:5.2f} slope {sl:+.2f}", flush=True)


if __name__ == "__main__":
    main(sys.argv[1:])
