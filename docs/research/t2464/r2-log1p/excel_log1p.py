"""Best model of Excel's ln(1+a) for -0.375 <= a < 0 (the log1p behind BETA.DIST's density and BINOM.DIST(0,..)),
T-2464 r2-log1p. Every op is an x87 op stored to double, fl53(fl64(.)):
    r  = a / (a + 2)                        (TOMS 708 alnrel's variable; the quotient is double-rounded)
    t2 = r * r
    s  = sum_{k=1..14} RN(1/(2k+1)) * (r * t2^k), terms formed by a running product p = p*t2, summed largest first
    l  = 2 * (r + s)
and LN(fl(1+a)) for a < -0.375 (the brcomp/alnrel switch; equal at a = -0.375)."""
from x87f import x87 as X

CS = [1.0 / (2 * k + 1) for k in range(15)]


def log1p_series(a: float, N: int = 14) -> float:
    r = X("/", a, a + 2.0)
    t2 = X("*", r, r)
    p = r
    s = None
    for k in range(1, N + 1):
        p = X("*", p, t2)
        tm = X("*", p, CS[k])
        s = tm if s is None else X("+", s, tm)
    return 2 * X("+", r, s)


def ln1m(x: float) -> float:
    """Excel's ln(1-x) for 0 < x < 1 as BETA.DIST's density and BINOM.DIST(0,n,p) use it."""
    if x <= 0.375:
        return log1p_series(-x)
    from cands import ln_fl  # Excel's LN(fl(1-x)), x87 FYL2X model

    return ln_fl(-x)
