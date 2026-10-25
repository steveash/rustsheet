"""Matched filter: correlate a block's residual r = D - dX (rmsfit) with the rounding error (in ulps, -0.5..0.5) of
candidate intermediates Q, each formed as a stored x87 op of doubles.  |rho| near 1/sqrt(N) is chance.
Usage: mfilter.py [--src S] BLOCKREGEX 'knobs'"""
import math
import re
import sys

import mpmath
import numpy as np

import r3lib as L
import rmsfit as RF
import scan as S

LD = L.LDB


def rerr(exact, stored: float) -> float:
    u = math.ulp(stored)
    return float((mpmath.mpf(stored) - exact) / u)


def cands(x, a, b):
    k, n, nk, ab = L.shape(a, b)
    X = mpmath.mpf(x)
    q = L.st(1 - LD(x))
    npx = L.st(LD(n) * LD(x))
    nqx = L.st(LD(n) * LD(q))
    c = {}

    def op(name, A, B, o):
        A_, B_ = mpmath.mpf(A), mpmath.mpf(B)
        ex = {"*": A_ * B_, "/": A_ / B_, "+": A_ + B_, "-": A_ - B_}[o]
        c[name] = rerr(ex, float(mpmath.mpf(L.DL.D(ex))))

    op("n*x", n, x, "*")
    op("n*q", n, q, "*")
    op("ab*x", ab, x, "*")
    op("ab*q", ab, q, "*")
    op("a*x", a, x, "*")
    op("b*q", b, q, "*")
    op("k*x", k, x, "*")
    op("nk*q", nk, q, "*")
    op("b*x", b, x, "*")
    op("a*q", a, q, "*")
    op("nk*x", nk, x, "*")
    op("k*q", k, q, "*")
    for side, kk, M in (("p", k, npx), ("q", nk, nqx)):
        d = kk - M
        sm = L.st(LD(kk) + LD(M))
        op(side + ".k+M", kk, M, "+")
        v = L.st(LD(d) / LD(sm))
        op(side + ".v", d, sm, "/")
        op(side + ".d*v", d, v, "*")
        op(side + ".2k*v", 2 * kk, v, "*")
        op(side + ".2M*v", 2 * M, v, "*")
        op(side + ".sm*v", sm, v, "*")
        op(side + ".n*v", n, v, "*")
        ve = mpmath.mpf(d) / sm
        c[side + ".2k*ve"] = rerr(2 * kk * ve, float(mpmath.mpf(L.DL.D(2 * kk * ve))))
        c[side + ".2M*ve"] = rerr(2 * M * ve, float(mpmath.mpf(L.DL.D(2 * M * ve))))
        c[side + ".2kd/sm"] = rerr(2 * kk * mpmath.mpf(d) / sm, float(mpmath.mpf(L.DL.D(2 * kk * mpmath.mpf(d) / sm))))
        op(side + ".v*v", v, v, "*")
        op(side + ".k/M", kk, M, "/")
        op(side + ".M/k", M, kk, "/")
        op(side + ".d/M", d, M, "/")
        op(side + ".d/k", d, kk, "/")
        op(side + ".d*d", d, d, "*")
        Tx = mpmath.mpf(kk) * mpmath.log(mpmath.mpf(kk) / M)
        c[side + ".T"] = rerr(Tx, float(mpmath.mpf(L.DL.D(Tx))))
        Tv = 2 * mpmath.mpf(kk) * mpmath.atanh(mpmath.mpf(v))
        c[side + ".Tv"] = rerr(Tv, float(mpmath.mpf(L.DL.D(Tv))))
        bd = Tx - d
        c[side + ".bd0"] = rerr(bd, float(mpmath.mpf(L.DL.D(bd))))
        tail = Tv - 2 * mpmath.mpf(kk) * mpmath.mpf(v)
        c[side + ".tail"] = rerr(tail, float(mpmath.mpf(L.DL.D(tail))))
        r = L.st(LD(kk) / LD(M))
        lnr = L.LN(r)
        op(side + ".k*LN", kk, lnr, "*")
    c["x"] = 0.0
    return c


def main(argv):
    src = "old"
    if argv[0] == "--src":
        src, argv = argv[1], argv[2:]
    rows = S.load(src)
    pat = re.compile(argv[0])
    o = S.parse(argv[1] if len(argv) > 1 else "")
    for blk, idx in sorted(L.blocks(rows).items()):
        if not pat.fullmatch(blk) or len(idx) < 64:
            continue
        Ds = RF.block_D(rows, idx)
        res = []
        F = []
        for i, (d, xe) in zip(idx, Ds):
            b1, b2, _ = L.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], o)
            xm = L.frac(-b1 - b2)
            res.append(d - float((mpmath.mpf(xm.numerator) / xm.denominator - xe) * RF.U))
            F.append(cands(rows[i]["x"], rows[i]["a"], rows[i]["b"]))
        r = np.array(res)
        r = r - r.mean()
        names = list(F[0])
        out = []
        for nm in names:
            f = np.array([z[nm] for z in F])
            if np.std(f) < 1e-9:
                continue
            rho = np.corrcoef(r, f)[0, 1]
            amp = np.dot(r, f - f.mean()) / np.dot(f - f.mean(), f - f.mean())
            out.append((abs(rho), nm, rho, amp))
        out.sort(reverse=True)
        print(f"{blk} sd(r) {np.std(r):.2f}  N={len(r)}  chance |rho|~{1/math.sqrt(len(r)):.3f}")
        for _, nm, rho, amp in out[:8]:
            print(f"    {nm:10s} rho {rho:+.3f}  amp {amp:+8.2f} units/ulp")


if __name__ == "__main__":
    main(sys.argv[1:])
