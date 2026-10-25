"""Verifier's own implementation of the r3 (Ce) candidate for Excel's Loader density, a, b > 1.

x87 arithmetic: numpy longdouble (64-bit significand, round to nearest); `st` stores to a double.
LN: Excel's LN of a double, from the engine's bit-exact model (excelmath.excel('LN', ...)), batched.
EXP: model V6 written here from its description (t = RN64(E * RN64(log2 e)), n = nint(t), f = t - n,
u = RN(2^|f| - 1), w = RN64(1 + u), w = RN64(1/w) if f < 0, result RN53(w 2^n)); checked against the engine's EXP on
doubles in check_exp().

Model (worker's description):
  k = fl(a-1), n = fl(fl(a+b)-2), nk = fl(n-k), q = fl(1-x), M = fl(n x), Mq = fl(n q)
  bd0(k, M): d = fl(k-M), sm = fl(k+M);
    |d| < 0.2 sm: series, v = fl(d/sm) (knob v='x'; 'e' keeps it extended), s = d v, ej = 2k v, minus
                  ph * (fl(2kv) - 2kv); loop ej *= v^2, s += ej/(2j+1) until s stops changing (all extended)
    else:         t = fl(k * LN(fl(k/M))); bd0 = t + fl(M-k)  (knob ld='x'; 'e' keeps M-k extended;
                  exploratory: 'xx' fl(t + fl(M-k)), 'R' fl(fl(t+M)-k); Xs=1 stores X on ll cells)
  X = -bd0(k, M) - bd0(nk, Mq);  E = C + X extended (chain Ce);  out = EXP_V6(E)
"""
from __future__ import annotations

import sys
from pathlib import Path

import mpmath
import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
import excelmath as XM  # noqa: E402

mpmath.mp.prec = 200
LD = np.longdouble
assert np.finfo(LD).nmant == 63, "needs x87 80-bit longdouble"


def st(v) -> LD:
    return LD(float(v))


def ld_of(m) -> LD:
    m = mpmath.mpf(m)
    if m == 0:
        return LD(0)
    s, man, ex, _ = m._mpf_
    return (LD(-1) if s else LD(1)) * LD(int(man)) * LD(2.0) ** int(ex)


def rn64(m) -> LD:
    return ld_of(XM.fl64(m))


L2E = rn64(1 / mpmath.log(2))
_F2: dict = {}


def exp_v6(E) -> float:
    t = LD(E) * L2E
    n = np.rint(t)
    f = t - n
    af = abs(f)
    key = af.as_integer_ratio()
    u = _F2.get(key)
    if u is None:
        u = _F2[key] = rn64(mpmath.mpf(2) ** (mpmath.mpf(key[0]) / key[1]) - 1)
    w = LD(1) + u
    if f < 0:
        w = LD(1) / w
    return float(np.ldexp(w, int(n)))


def check_exp(nn: int = 3000, seed: int = 7) -> int:
    rng = np.random.default_rng(seed)
    xs = [float(v) for v in rng.uniform(-40, 8, nn)]
    ref = XM.excel("EXP", xs)
    return sum(exp_v6(x) != r for x, r in zip(xs, ref))


class LNBatch:
    """Excel LN for doubles, evaluated through the engine in batches and memoised."""

    def __init__(self):
        self.m: dict = {}

    def need(self, vals):
        new = sorted({float(v) for v in vals} - self.m.keys())
        for i in range(0, len(new), 20000):
            chunk = new[i:i + 20000]
            self.m.update(zip(chunk, XM.excel("LN", chunk)))

    def __call__(self, v) -> float:
        return self.m[float(v)]


LN = LNBatch()


def shape(a, b):
    k = float(LD(a) - 1)
    ab = float(LD(a) + LD(b))
    n = float(LD(ab) - 2)
    nk = float(LD(n) - LD(k))
    return k, n, nk


def means(x, a, b):
    k, n, nk = shape(a, b)
    q = float(1 - LD(x))
    return (k, float(LD(n) * LD(x))), (nk, float(LD(n) * LD(q)))


def is_series(k, M, cut=0.2):
    d = st(LD(k) - LD(M))
    sm = st(LD(k) + LD(M))
    return abs(d) < LD(cut) * sm


def ratios_needed(x, a, b, cut=0.2):
    return [float(LD(k) / LD(M)) for k, M in means(x, a, b) if not is_series(k, M, cut)]


def bd0(k, M, o):
    K, Mm = LD(k), LD(M)
    d = st(K - Mm)
    sm = st(K + Mm)
    if abs(d) < LD(o.get("cut", 0.2)) * sm:
        v = d / sm
        if o.get("v", "x") == "x":
            v = st(v)
        s = d * v
        ej = LD(2) * K * v
        if o.get("ph", 1):
            s = s - (st(ej) - ej)
        v2 = v * v
        for j in range(1, 2000):
            ej = ej * v2
            s1 = s + ej / LD(2 * j + 1)
            if s1 == s:
                return s1, "s"
            s = s1
        return s, "s"
    r = float(K / Mm)
    t = st(K * LD(LN(r)))
    ld = o.get("ld", "x")
    if ld == "R":  # R's fl(fl(t + M) - k)
        return st(st(t + Mm) - K), "l"
    mk = st(Mm - K) if ld in ("x", "xx") else Mm - K
    return (st(t + mk) if ld == "xx" or o.get("bs") else t + mk), "l"


def xpart(x, a, b, o):
    (k, M), (nk, Mq) = means(x, a, b)
    b1, r1 = bd0(k, M, o)
    b2, r2 = bd0(nk, Mq, o)
    if o.get("Xs") and r1 + r2 == "ll":  # X = fl(-b1 - b2) stored (exploratory knob)
        return st(-b1 - b2), r1 + r2
    return -b1 - b2, r1 + r2


def exact_logpdf(x, a, b) -> LD:
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    return ld_of(XM.fl64((A - 1) * mpmath.log(X) + (B - 1) * mpmath.log1p(-X) - mpmath.log(mpmath.beta(A, B))))


def exact_pdf(x, a, b) -> float:
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    return float(X ** (A - 1) * (1 - X) ** (B - 1) / mpmath.beta(A, B))
