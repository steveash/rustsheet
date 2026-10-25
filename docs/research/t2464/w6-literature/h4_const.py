"""w6-literature H4: approximation-error fingerprints of published ln B / ln Gamma approximants.

If Excel's a, b > 2 density were exp(E) evaluated accurately (extended) but with a library's ln B approximant, the
block offset (Excel - CR, units of 2^-53 relative) would equal -(lnB_lib - lnB_exact)/2^-53, the same on both sides
of the mean. This evaluates each approximant in 100-digit arithmetic with the source's own coefficients (so only
its approximation error and its decimal constants remain), and prints the predicted offsets next to Excel's
measured block means (from h2b_resid.out / w2-forensics).
Sources: TOMS 708 betaln/gamln/gamln1/algdiv/bcorr (R toms708.c), R lbeta + lgammacor, Loader 2000 stirlerr table.
"""

import mpmath

mpmath.mp.dps = 100
M = mpmath.mpf

P = [
    0.577215664901533,
    0.844203922187225,
    -0.168860593646662,
    -0.780427615533591,
    -0.402055799310489,
    -0.0673562214325671,
    -0.00271935708322958,
]
Q = [
    1.0,
    2.88743195473681,
    3.12755088914843,
    1.56875193295039,
    0.361951990101499,
    0.0325038868253937,
    6.67465618796164e-4,
]
Rr = [
    0.422784335098467,
    0.848044614534529,
    0.565221050691933,
    0.156513060486551,
    0.017050248402265,
    4.97958207639485e-4,
]
Ss = [1.0, 1.24313399877507, 0.548042109832463, 0.10155218743983, 0.00713309612391, 1.16165475989616e-4]
C = [
    0.0833333333333333,
    -0.00277777777760991,
    7.9365066682539e-4,
    -5.9520293135187e-4,
    8.37308034031215e-4,
    -0.00165322962780713,
]


def poly(c, x):
    return sum(M(ci) * x**i for i, ci in enumerate(c))


def gamln1(a):
    a = M(a)
    if a < 0.6:
        return -a * poly(P, a) / poly(Q, a)
    x = a - 1
    return x * poly(Rr, x) / poly(Ss, x)


def gamln(a):
    a = M(a)
    if a <= 0.8:
        return gamln1(a) - mpmath.log(a)
    if a <= 2.25:
        return gamln1(a - 1)
    if a < 10:
        n = int(a - 1.25)
        t, w = a, M(1)
        for _ in range(n):
            t -= 1
            w *= t
        return gamln1(t - 1) + mpmath.log(w)
    t = 1 / (a * a)
    w = sum(M(c) * t**i for i, c in enumerate(C)) / a
    return M(0.418938533204673) + w + (a - 0.5) * (mpmath.log(a) - 1)


def series_w(a, b):
    h = a / b
    c = h / (h + 1)
    x = 1 / (h + 1)
    x2 = x * x
    s3 = 1 + (x + x2)
    s5 = 1 + (x + x2 * s3)
    s7 = 1 + (x + x2 * s5)
    s9 = 1 + (x + x2 * s7)
    s11 = 1 + (x + x2 * s9)
    t = (1 / b) ** 2
    w = (
        (((M(C[5]) * s11 * t + M(C[4]) * s9) * t + M(C[3]) * s7) * t + M(C[2]) * s5) * t + M(C[1]) * s3
    ) * t + M(C[0])
    return w * c / b


def algdiv(a, b):
    a, b = M(a), M(b)
    w = series_w(a, b)
    d = b + (a - 0.5)
    u = d * mpmath.log1p(a / b)
    v = a * (mpmath.log(b) - 1)
    return w - v - u


def bcorr(a0, b0):
    a, b = min(M(a0), M(b0)), max(M(a0), M(b0))
    w = series_w(a, b)
    t = (1 / a) ** 2
    return sum(M(c) * t**i for i, c in enumerate(C)) / a + w


def betaln(a0, b0):
    a, b = min(M(a0), M(b0)), max(M(a0), M(b0))
    if a >= 8:
        w = bcorr(a, b)
        h = a / b
        c = h / (h + 1)
        u = -(a - 0.5) * mpmath.log(c)
        v = b * mpmath.log1p(h)
        return mpmath.log(b) * -0.5 + M(0.918938533204673) + w - u - v
    if a < 1:
        return gamln(a) + (gamln(b) - gamln(a + b)) if b < 8 else gamln(a) + algdiv(a, b)
    if a <= 2:
        if b <= 2:
            return gamln(a) + gamln(b) - gamln(a + b)
        if b < 8:
            return l60(a, b, M(0))
        return gamln(a) + algdiv(a, b)
    n = int(a - 1)
    w = M(1)
    for _ in range(n):
        a -= 1
        h = a / b
        w *= h / (h + 1)
    w = mpmath.log(w)
    if b >= 8:
        return w + gamln(a) + algdiv(a, b)
    return l60(a, b, w)


def l60(a, b, w):
    n = int(b - 1)
    z = M(1)
    for _ in range(n):
        b -= 1
        z *= b / (a + b)
    return w + mpmath.log(z) + (gamln(a) + (gamln(b) - gamln(a + b)))


ALG = [
    +0.1666389480451863247205729650822e0,
    -0.1384948176067563840732986059135e-4,
    +0.9810825646924729426157171547487e-8,
    -0.1809129475572494194263306266719e-10,
    +0.6221098041892605227126015543416e-13,
]


def lgammacor(x):
    x = M(x)
    t = 2 * (10 / x) ** 2 - 1
    # Chebyshev sum with the R convention (first coefficient halved)
    T = [M(1), t]
    for _ in range(3):
        T.append(2 * t * T[-1] - T[-2])
    return (sum(M(c) * T[i] for i, c in enumerate(ALG)) - M(ALG[0]) / 2) / x


def r_lbeta(a, b):
    p, q = min(M(a), M(b)), max(M(a), M(b))
    if p >= 10:
        corr = lgammacor(p) + lgammacor(q) - lgammacor(p + q)
        return (
            mpmath.log(q) * -0.5
            + M(0.918938533204672741780329736406)
            + corr
            + (p - 0.5) * mpmath.log(p / (p + q))
            + q * mpmath.log1p(-p / (p + q))
        )
    if q >= 10:
        corr = lgammacor(q) - lgammacor(p + q)
        return mpmath.loggamma(p) + corr + p - p * mpmath.log(p + q) + (q - 0.5) * mpmath.log1p(-p / (p + q))
    return mpmath.log(mpmath.beta(p, q))


U = M(2) ** -53
EXCEL = {  # block mean of (Excel - CR)/2^-53, from h2b_resid.out (pdf)
    (2.5, 3.5): "+2.7..+3.4 (both sides of the mean)",
    (4.0, 4.0): "-2.40",
    (7.3, 11.6): "-4.2 (x .273) / +5.4 (x .613); a-sweep +1.1, b-sweep +1.0",
    (12.0, 20.0): "-1.0 (sd 21)",
    (100.0, 120.0): "-1.1 (sd 13)",
    (10.0, 2.5): "+6.3 (sd 49)",
}

if __name__ == "__main__":
    print("(a,b): predicted density offset = -(lnB_lib - lnB_exact)/2^-53 ; Excel measured")
    for (a, b), meas in EXCEL.items():
        ex = mpmath.log(mpmath.beta(M(a), M(b)))
        t = -(betaln(a, b) - ex) / U
        r = -(r_lbeta(a, b) - ex) / U
        print(f"  ({a:g},{b:g}): TOMS betaln {float(t):+8.2f}   R lbeta {float(r):+8.2f}   | Excel {meas}")
    print("TOMS gamln(v) - lnGamma(v), units of 2^-53 absolute:")
    for v in [0.25, 0.5, 0.75, 1.5, 2.5, 3.5, 4, 6, 7.3, 11.6, 18.9, 12, 20, 32]:
        print(f"  {v:g}: {float((gamln(v) - mpmath.loggamma(M(v))) / U):+.3f}")
