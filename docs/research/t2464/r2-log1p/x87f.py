"""Fast exact x87 stored ops on doubles: fl53(fl64(a op b)) via integer arithmetic (Fractions-free)."""
import math


def _round(num: int, den: int, bits: int):
    """Round num/den (den>0, num != 0) to `bits` significant bits, ties to even; returns (mant, exp) with value
    mant*2^exp."""
    neg = num < 0
    num = abs(num)
    e = num.bit_length() - den.bit_length()
    # want q = floor(num/den * 2^(bits - e)) with bits or bits+1 bits
    sh = bits - e
    n2, d2 = (num << sh, den) if sh >= 0 else (num, den << -sh)
    q, rem = divmod(n2, d2)
    if q.bit_length() > bits:
        sh -= 1
        n2, d2 = (num << sh, den) if sh >= 0 else (num, den << -sh)
        q, rem = divmod(n2, d2)
    two = 2 * rem
    if two > d2 or (two == d2 and q & 1):
        q += 1
    if q.bit_length() > bits:
        q >>= 1
        sh -= 1
    return (-q if neg else q), -sh


def _rat(x: float):
    m, e = math.frexp(x)
    M = int(m * 2**53)
    return M, e - 53


def x87(op: str, a: float, b: float) -> float:
    """fl53(fl64(a op b)) for finite doubles (normal range)."""
    ma, ea = _rat(a)
    mb, eb = _rat(b)
    if op == "*":
        num, e, den = ma * mb, ea + eb, 1
    elif op == "/":
        num, e, den = ma, ea - eb, mb
        if den < 0:
            num, den = -num, -den
    else:
        if op == "-":
            mb = -mb
        e = min(ea, eb)
        num, den = (ma << (ea - e)) + (mb << (eb - e)), 1
    if num == 0:
        return 0.0
    if e >= 0:
        num <<= e
        e2 = 0
    else:
        e2 = e
    m64, x64 = _round(num, den, 64)
    m53, x53 = _round(m64, 1, 53)
    return math.ldexp(m53, x53 + x64 + e2)


def xd(op, a, b):
    return x87(op, a, b)
