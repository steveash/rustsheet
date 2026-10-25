"""TOMS 708 algdiv(a, b) = ln(Gamma(b) / Gamma(a+b)) (b >= 8) and betaln's a < 1 / 1 <= a < 2 branches with a
pluggable arithmetic, to find Excel's variant. Each variant is a dict of switches:

  ar     "d" every op in double; "x" every op an x87 op stored to double; "e" every op extended (64-bit), stored only
         where `st` says
  st     which named intermediates are stored to double under ar="e" (subset of h c x d s w u v r)
  aln    alnrel(a/b): "toms" (3,3) rational in t = a/(a+2); "ser" r2-log1p's 14-term atanh series; "cr" correctly
         rounded log1p; "ln" LN(fl(1 + a/b))
  lnb    log(b): "xl" Excel's LN (double); "e" FYL2X extended (unstored)
  t2     t = (1/b)^2 ("sq") or 1/(b*b) ("bb")
  fin    final combination: "toms" (w - v - u if u > v else w - u - v), "wuv" ((w - u) - v), "wvu", "w(uv)"
         (w - (u + v))
  gsum   lnB = GL(a) + algdiv: "d" stored, "e" extended
The values are mpf (extended where unstored); the caller rounds lnB to double unless gsum="e".
"""

from __future__ import annotations

import mpmath

import m1lib as M

mpf = M.mpf
R53, R64 = M.fl53, M.fl64
C0, C1, C2, C3, C4, C5 = (
    0.0833333333333333,
    -0.00277777777760991,
    7.9365066682539e-4,
    -5.9520293135187e-4,
    8.37308034031215e-4,
    -0.00165322962780713,
)
P = (-1.29418923021993, 0.405303492862024, -0.0178874546012214)
Q = (-1.62752256355323, 0.747811014037616, -0.0845104217945565)


class Ar:
    def __init__(self, ar: str, st: str = ""):
        self.ar, self.st = ar, st

    def r(self, v, name=""):
        if self.ar == "d":
            return mpf(R53(v))
        if self.ar == "x":
            return mpf(R53(R64(v)))
        v = R64(v)
        return mpf(R53(v)) if name and name in self.st else v


def alnrel_pos(q, mode, A: Ar):
    """ln(1 + q), q > 0 small (q = a/b <= 1/8 here)."""
    if mode == "cr":
        return mpf(float(mpmath.log1p(q)))
    if mode == "ln":
        return mpf(M.BE.LN(float(R53(1 + q))))
    if mode == "ser":
        return mpf(M.XLP.log1p_series(float(q)))
    t = A.r(q / A.r(q + 2))
    t2 = A.r(t * t)
    num = A.r(A.r(A.r(A.r(A.r(P[2] * t2) + P[1]) * t2) + P[0]) * t2) + 1
    num = A.r(num)
    den = A.r(A.r(A.r(A.r(A.r(Q[2] * t2) + Q[1]) * t2) + Q[0]) * t2) + 1
    den = A.r(den)
    return A.r(A.r(t * 2) * A.r(num / den), "r")


def algdiv(a: float, b: float, v: dict):
    A = Ar(v.get("ar", "d"), v.get("st", ""))
    a_, b_ = mpf(a), mpf(b)
    if a > b:
        h = A.r(b_ / a_, "h")
        c = A.r(1 / A.r(h + 1), "c")
        x = A.r(h / A.r(h + 1), "x")
        d = A.r(a_ + A.r(b_ - 0.5), "d")
    else:
        h = A.r(a_ / b_, "h")
        c = A.r(h / A.r(h + 1), "c")
        x = A.r(1 / A.r(h + 1), "x")
        d = A.r(b_ + A.r(a_ - 0.5), "d")
    x2 = A.r(x * x)
    s = []
    acc = mpf(1)
    for _ in range(5):
        acc = A.r(A.r(x + A.r(x2 * acc)) + 1, "s")
        s.append(acc)
    s3, s5, s7, s9, s11 = s
    if v.get("t2", "sq") == "bb":
        t = A.r(1 / A.r(b_ * b_))
    else:
        t = A.r(1 / b_)
        t = A.r(t * t)
    w = A.r(A.r(C5 * s11) * t)
    w = A.r(A.r(A.r(w + A.r(C4 * s9)) * t))
    w = A.r(A.r(w + A.r(C3 * s7)) * t)
    w = A.r(A.r(w + A.r(C2 * s5)) * t)
    w = A.r(A.r(w + A.r(C1 * s3)) * t)
    w = A.r(w + C0)
    w = A.r(w * A.r(c / b_), "w")
    q = A.r(a_ / b_)
    al = alnrel_pos(q, v.get("aln", "toms"), A)
    u = A.r(d * al, "u")
    lb = mpf(M.BE.LN(b)) if v.get("lnb", "xl") == "xl" else M.W1.ln64(b)
    vv = A.r(a_ * A.r(lb - 1), "v")
    fin = v.get("fin", "toms")
    if fin == "toms":
        fin = "wvu" if u > vv else "wuv"
    if fin == "wvu":
        return A.r(A.r(w - vv) - u)
    if fin == "wuv":
        return A.r(A.r(w - u) - vv)
    return A.r(w - A.r(u + vv))


def lnb(a: float, b: float, v: dict):
    """betaln for min(a, b) <= 1 < 8 <= max (M1's branch): GL(min) + algdiv(min, max)."""
    lo, hi = min(a, b), max(a, b)
    g = mpf(M.BE.GAMMALN(lo)) if lo != 1.0 else mpf(0)
    s = g + algdiv(lo, hi, v)
    return R64(s) if v.get("gsum", "d") == "e" else mpf(R53(R64(s)))


def algdiv_semi(a: float, b: float, rnd: str, aln: str = "cr"):
    """TOMS algdiv in exact (200-bit) arithmetic with exact log/log1p, except the named pieces rounded to double:
    'l' LN(b) (Excel's), 'q' a/b, 'd' d = b + (a - 0.5), 'r' alnrel(a/b) (aln: 'cr' or 'toms' rational in
    double), 'u', 'v', 'w', 'f' the final value."""
    A, B = mpf(a), mpf(b)
    h = A / B
    if "q" in rnd:
        h = mpf(R53(h))
    c, x = h / (h + 1), 1 / (h + 1)
    d = B + (A - mpf(0.5))
    if "d" in rnd:
        d = mpf(R53(d))
    x2 = x * x
    s, acc = [], mpf(1)
    for _ in range(5):
        acc = x + x2 * acc + 1
        s.append(acc)
    t = 1 / (B * B)
    w = ((((C5 * s[4] * t + C4 * s[3]) * t + C3 * s[2]) * t + C2 * s[1]) * t + C1 * s[0]) * t + C0
    w = w * (c / B)
    if "w" in rnd:
        w = mpf(R53(w))
    if "r" in rnd:
        al = alnrel_pos(mpf(R53(h)), aln, Ar("d")) if aln == "toms" else mpf(float(mpmath.log1p(h)))
    else:
        al = mpmath.log1p(h)
    u = d * al
    if "u" in rnd:
        u = mpf(R53(u))
    lb = mpf(M.BE.LN(b)) if "l" in rnd else mpmath.log(B)
    v = A * (lb - 1)
    if "v" in rnd:
        v = mpf(R53(v))
    r = w - u - v
    return mpf(R53(r)) if "f" in rnd else r
