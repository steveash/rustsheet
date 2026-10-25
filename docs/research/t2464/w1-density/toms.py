"""TOMS 708 pieces (R toms708.c coefficients) in plain double and in x87-stored arithmetic (mpmath)."""
import math
import mpmath
from lib import D, R64, mpf

P1, P2, P3 = -0.129418923021993e01, 0.405303492862024e00, -0.178874546012214e-01
Q1, Q2, Q3 = -0.162752256355323e01, 0.747811014037616e00, -0.845104217945565e-01


def alnrel_double(a: float) -> float:
    """ln(1+a), TOMS 708 alnrel, IEEE double arithmetic (|a| <= 0.375 branch)."""
    t = a / (a + 2.0)
    t2 = t * t
    w = (((P3 * t2 + P2) * t2 + P1) * t2 + 1.0) / (((Q3 * t2 + Q2) * t2 + Q1) * t2 + 1.0)
    return t * 2.0 * w


def alnrel_x87(a, store=True):
    """Same with every op rounded fl53(fl64()) (store=True) or kept in 64-bit (store=False), result stored."""
    S = D if store else R64
    a = mpf(a)
    t = S(a / S(a + 2))
    t2 = S(t * t)
    n = S(S(S(S(S(S(mpf(P3) * t2) + P2) * t2) + P1) * t2) + 1)
    d = S(S(S(S(S(S(mpf(Q3) * t2) + Q2) * t2) + Q1) * t2) + 1)
    w = S(n / d)
    return D(S(t * 2) * w)
