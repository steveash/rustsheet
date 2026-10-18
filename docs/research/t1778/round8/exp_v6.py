# Copied by T-1916 from branch agent/research/T-1778-kernel-map at 59c2c3724a3c (path docs/research/t1778/round8/exp_v6.py), unchanged below this line.
"""Excel EXP model V6 (T-1778 round 8): 8087-domain F2XM1 with a reciprocal for negative fractions.

FLDL2E; FMUL a (64-bit, RN) -> t; FRNDINT (RN, ties-even) -> n; f = t - n (exact, |f| <= 1/2);
u = F2XM1(|f|) (modelled as RN64(2^|f| - 1)); w = RN64(1 + u); if f < 0: w = RN64(1 / w);
EXP = RN53(w * 2^n) (gradual underflow, single rounding).

Usage: python3 exp_v6.py FILE, where FILE has lines "a_hex E_hex [tag]". For EXP(-(x*x)) cells, a = -RN53(RN64(x*x)).
"""

import sys

import mpmath as mp

mp.mp.prec = 256


def _r64(v: mp.mpf) -> mp.mpf:
    with mp.workprec(64):
        return +v


_L2E64 = _r64(1 / mp.log(2))


def excel_exp(a: float) -> float:
    t = _r64(mp.mpf(a) * _L2E64)
    n = mp.nint(t)  # round half to even
    f = t - n  # exact
    u = _r64(mp.power(2, abs(f)) - 1)
    w = _r64(1 + u)
    if f < 0:
        w = _r64(1 / w)
    return float(mp.ldexp(w, int(n)))


if __name__ == "__main__":
    ok = n = 0
    for line in open(sys.argv[1]):
        a, e = (float.fromhex(s) for s in line.split()[:2])
        n += 1
        ok += excel_exp(a) == e
    print(f"{ok}/{n}")
