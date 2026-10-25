"""T.DIST(0, n, FALSE) against parameter-free Loader constants: C = -bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) -
stirlerr(n/2) with R's stirlerr (table / series, w6 h2_loader SF), bd0 by r3lib (x87 knobs) or exact; output chains
EXP(C)/sqrt(2 pi) variants.  Prints ulps of each candidate from Excel.  usage: s15_t0.py"""
import sys

import mpmath

import tdlib as TD

sys.path.insert(0, str(TD.T / "w6-literature"))
from h2_loader import S0, S1, S2, S3, S4, SF  # noqa: E402

L, st = TD.LDB, TD.st
XL = {1: 0.3183098861837907, 2: 0.3535533905932738, 3: 0.3675525969478615, 4: 0.37499999999999994,
      5: 0.3796066898224945, 10: 0.38910838396603115, 30: 0.3956321848940978, 100: 0.39794618693589384,
      1000: 0.39884255731385815, 1e6: 0.39894218066587506}  # m9-t-dist-pdf (cells.tsv)


def stir(v, ar="x"):
    if v <= 15 and (2 * v) == int(2 * v):
        return L(SF[int(2 * v)])
    R = (lambda z: L(st(z))) if ar == "x" else (lambda z: z)
    V = L(v)
    nn = R(V * V)
    if v > 500:
        return R(R(L(S0) - R(L(S1) / nn)) / V)
    if v > 80:
        return R(R(L(S0) - R(R(L(S1) - R(L(S2) / nn)) / nn)) / V)
    if v > 35:
        return R(R(L(S0) - R(R(L(S1) - R(R(L(S2) - R(L(S3) / nn)) / nn)) / nn)) / V)
    return R(R(L(S0) - R(R(L(S1) - R(R(L(S2) - R(R(L(S3) - R(L(S4) / nn)) / nn)) / nn)) / nn)) / V)


def consts(n):
    k, M = L(n / 2), L((n + 1) / 2)
    out = {}
    for bn, bo in (("bx", {"v": "x", "s": "x", "ej": "x", "t": "x", "v2": "x"}), ("be", {}), ("bph", {"v": "x", "ph": 1})):
        b, _ = TD.R3.bd0(k, M, bo)
        s1, s2 = stir((n + 1) / 2), stir(n / 2)
        out[f"{bn},((-b+s1)-s2)x"] = L(st(L(st(-b + s1)) - s2))
        out[f"{bn},e"] = -b + s1 - s2
        out[f"{bn},(s1-s2)-b x"] = L(st(L(st(s1 - s2)) - b))
    out["exact"] = TD.ld_of(TD.loader_const_exact(n))
    return out


S2PI = L(float(mpmath.sqrt(2 * mpmath.pi)))
for n, xv in XL.items():
    if n < 3:
        continue
    row = []
    for cn, C in consts(n).items():
        for chain in ("old", "mul", "ext"):
            if chain == "old":
                v = st(L(TD.exp_v6(L(st(C)))) / L(st(np_sqrt := L(st(L(TD.TWO_PI))) ** 0.5)))
            elif chain == "mul":
                v = st(L(TD.exp_v6(L(st(C)))) * L(TD.INV_SQRT_2PI))
            else:
                v = st(TD.exp_v6(C - L(TD.LN_SQRT_2PI), True))
            row.append(f"{cn}/{chain}:{(v - xv) / __import__('math').ulp(xv):+.0f}")
    cr = float(mpmath.exp(TD.exact_logpdf(0.0, n)))
    print(f"n={n:g} CR {(cr - xv) / __import__('math').ulp(xv):+.0f} | " + " ".join(row))
