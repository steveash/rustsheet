"""Regularized incomplete beta I_x(a, b) (T-1744, rank 3 in docs/research/numeric-kernel-map.md).

Fits over the T-1044/T-1047/T-1048/T-1049 sweeps (probes-numeric, excel-graph):
- `beta-dist`: BETA.DIST(x,a,b,TRUE) and BETADIST(x,a,b), a, b in {0.0001 … 100}, x in {0.1, 0.5,
  0.9} plus the legacy grid: I_x(a, b) itself.
- `f-dist`: F.DIST(x,d1,d2,TRUE) = I_y(d1/2, d2/2) with y = d1*x/(d1*x + d2).
- `t-dist`: T.DIST(t,n,TRUE) = 1 - I_y(n/2, 1/2)/2 for t > 0 (I_y/2 for t < 0), y = n/(n + t^2).

Candidates, each plain IEEE double (Python floats, platform libm):
- Numerical Recipes `betacf` (modified Lentz) behind the log-space prefactor
  exp(lgamma(a+b) - lgamma(a) - lgamma(b) + a ln x + b ln(1-x)), switching at (a+1)/(a+b+2);
- Cephes `incbet` as published (power series when b*x <= 1 and x <= 0.95, else the two continued
  fractions, prefactor pow(1-x,b)*pow(x,a)/a*w*Gamma(a+b)/(Gamma(a)Gamma(b))), with x^a by `pow`
  or exp(a ln x), and Gamma by libm `tgamma` or correctly rounded;
- the best combination found: Cephes' continued fractions only (no power series), switching at the
  mean a/(a+b), prefactor exp(a ln x)*exp(b ln(1-x))*G/a*w with G correctly rounded, and Excel's
  closed forms I_x(a,1) = exp(a ln x) and I_x(1,b) = 1 - exp(b ln(1-x));
- the correctly rounded I_x(a, b) (70-digit decimal continued fraction), as a reference.

Result (T-1744 Outcome; docs/excel-semantics/statistics.md, "Incomplete beta: Excel's closed forms
and the best candidate"): no candidate is Excel's algorithm. I_x(a,1) = exp(a ln x) holds on 27 of
27 b = 1 cells; I_x(1,b) = 1 - exp(b ln(1-x)) on 22 of 24 a = 1 cells (1 - pow(1-x,b) equally).
The best combination is bit-exact on 158 of the 283 BETA.DIST cells (279 numeric), against 87 for
the engine (statrs `beta_reg`) and 130 for the correctly rounded value.
"""

from __future__ import annotations

import decimal
import math
import sys
from collections.abc import Callable
from fractions import Fraction
from functools import cache

from rustsheet_tools.fitlab.exact import pi_cr
from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, numeric_arg

_BETA_CASES = [
    f"probes-numeric/m9-beta-dist-a{a}" for a in ("0p0001", "0p5", "1", "2", "2p5", "5", "10", "50", "100")
] + ["probes-numeric/m9-legacy-betadist"]
_BETA_PATTERN = (
    r"^=(?:BETA\.DIST\(([^,()]+),([^,()]+),([^,()]+),TRUE\)|BETADIST\(([^,()]+),([^,()]+),([^,()]+)\))$"
)
_F_CASES = ["probes-numeric/m9-f-dist", "probes-numeric/m9-legacy-fdist", "probes-numeric/t1049-fdist"]
_F_PATTERN = r"^=F\.DIST\(([^,()]+),([^,()]+),([^,()]+),TRUE\)$"
_T_CASES = ["probes-numeric/m9-t-dist-cdf", "probes-numeric/t1048-tdist"]
_T_PATTERN = r"^=T\.DIST\(([^,()]+),([^,()]+),TRUE\)$"

Ibeta = Callable[[float, float, float], float]  # (x, a, b) -> I_x(a, b)

# The reference: a 70-digit decimal context, ln Gamma by Stirling after shifting z above 60.
# Decimal rather than `exact` Fractions: ln Gamma and a continued fraction at 70 digits per cell
# are far cheaper this way.
_CTX = decimal.Context(prec=70)
_PI_FRACTION = pi_cr(280)  # about 84 digits, beyond the context
_PI = _CTX.divide(decimal.Decimal(_PI_FRACTION.numerator), decimal.Decimal(_PI_FRACTION.denominator))
_SHIFT = 60
_STIRLING_TERMS = 30
_MAX_CF_TERMS = 5000  # directive 5: the reference fraction converges in far fewer for a, b <= 1e3


@cache
def _bernoulli_even() -> list[Fraction]:
    """B_0 … B_{2*_STIRLING_TERMS} (Akiyama-Tanigawa)."""
    n = 2 * _STIRLING_TERMS
    row: list[Fraction] = [Fraction(0)] * (n + 1)
    out: list[Fraction] = []
    for m in range(n + 1):
        row[m] = Fraction(1, m + 1)
        for j in range(m, 0, -1):
            row[j - 1] = j * (row[j - 1] - row[j])
        out.append(row[0])
    return out


def ln_gamma_decimal(z: decimal.Decimal) -> decimal.Decimal:
    """ln Gamma(z), z > 0, to about 60 digits."""
    c = _CTX
    shift = decimal.Decimal(0)
    steps = 0
    while z < _SHIFT and steps <= _SHIFT + 1:  # z > 0 reaches 60 within 61 steps
        shift = c.add(shift, c.ln(z))
        z = c.add(z, 1)
        steps += 1
    total = c.subtract(c.multiply(z - decimal.Decimal("0.5"), c.ln(z)), z) + c.ln(2 * _PI) / 2
    bern = _bernoulli_even()
    power = z
    for k in range(1, _STIRLING_TERMS):
        b = bern[2 * k]
        term = c.divide(decimal.Decimal(b.numerator), decimal.Decimal(b.denominator))
        total = c.add(total, c.divide(term, c.multiply(2 * k * (2 * k - 1), power)))
        power = c.multiply(power, c.multiply(z, z))
    return c.subtract(total, shift)


def _ibeta_decimal(dx: decimal.Decimal, da: decimal.Decimal, db: decimal.Decimal) -> decimal.Decimal:
    """I_x(a, b) for 0 < x < 1 at 70 digits: NR's continued fraction."""
    c = _CTX
    flip = dx > (da + 1) / (da + db + 2)
    if flip:
        dx, da, db = 1 - dx, db, da
    lbt = ln_gamma_decimal(da + db) - ln_gamma_decimal(da) - ln_gamma_decimal(db)
    lbt = lbt + da * c.ln(dx) + db * c.ln(1 - dx)
    qab, qap, qam = da + db, da + 1, da - 1
    cc, d = decimal.Decimal(1), 1 / (1 - qab * dx / qap)
    h = d
    tol = decimal.Decimal(10) ** -62
    for m in range(1, _MAX_CF_TERMS):
        m2 = 2 * m
        aa = m * (db - m) * dx / ((qam + m2) * (da + m2))
        d = 1 / (1 + aa * d)
        cc = 1 + aa / cc
        h *= d * cc
        aa = -(da + m) * (qab + m) * dx / ((da + m2) * (qap + m2))
        d = 1 / (1 + aa * d)
        cc = 1 + aa / cc
        delta = d * cc
        h *= delta
        if abs(delta - 1) < tol:
            break
    v = c.exp(lbt) * h / da
    return 1 - v if flip else v


def ibeta_reference(x: float, a: float, b: float) -> float:
    """I_x(a, b) rounded once to a double: NR's continued fraction at 70 digits."""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    return float(_ibeta_decimal(decimal.Decimal(x), decimal.Decimal(a), decimal.Decimal(b)))


@cache
def gamma_cr(z: float) -> float:
    """Gamma(z) correctly rounded (from the decimal ln Gamma)."""
    return float(_CTX.exp(ln_gamma_decimal(decimal.Decimal(z))))


@cache
def ln_gamma_cr(z: float) -> float:
    """ln Gamma(z) correctly rounded."""
    return float(ln_gamma_decimal(decimal.Decimal(z)))


def gamma_ratio_cr(a: float, b: float) -> float:
    """Gamma(a+b)/(Gamma(a)*Gamma(b)) from correctly rounded Gammas (Cephes' form)."""
    return gamma_cr(a + b) / (gamma_cr(a) * gamma_cr(b))


def gamma_ratio_libm(a: float, b: float) -> float:
    """Gamma(a+b)/(Gamma(a)*Gamma(b)) from libm tgamma (Cephes' own source)."""
    return math.gamma(a + b) / (math.gamma(a) * math.gamma(b))


def exp_log_power(x: float, a: float) -> float:
    """x^a as exp(a*ln x): Excel's I_x(a,1) (all 27 BETA.DIST cells with b = 1)."""
    return math.exp(a * math.log(x))


def libm_pow(x: float, a: float) -> float:
    """x^a by libm pow."""
    return float(x**a)


# Numerical Recipes 3rd ed. `betacf` (modified Lentz) and `betai`.
_FPMIN = 1e-300
_MAX_NR_TERMS = 1000


def nr_betacf(a: float, b: float, x: float, eps: float = 3e-16) -> float:
    """Numerical Recipes' continued fraction for I_x(a,b), modified Lentz, to `eps`."""
    qab, qap, qam = a + b, a + 1.0, a - 1.0
    c = 1.0
    d = 1.0 - qab * x / qap
    d = 1.0 / (d if abs(d) >= _FPMIN else _FPMIN)
    h = d
    for m in range(1, _MAX_NR_TERMS + 1):
        m2 = 2 * m
        for aa in (
            m * (b - m) * x / ((qam + m2) * (a + m2)),
            -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2)),
        ):
            d = 1.0 + aa * d
            d = 1.0 / (d if abs(d) >= _FPMIN else _FPMIN)
            c = 1.0 + aa / c
            c = c if abs(c) >= _FPMIN else _FPMIN
            delta = d * c
            h *= delta
        if abs(delta - 1.0) < eps:
            break
    return h


def nr_betai(x: float, a: float, b: float, log1p_tail: bool = False, eps: float = 3e-16) -> float:
    """Numerical Recipes' betai: log-space prefactor, fraction, switch at (a+1)/(a+b+2)."""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    l1x = math.log1p(-x) if log1p_tail else math.log(1.0 - x)
    bt = math.exp(math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + a * math.log(x) + b * l1x)
    if x < (a + 1.0) / (a + b + 2.0):
        return bt * nr_betacf(a, b, x, eps) / a
    return 1.0 - bt * nr_betacf(b, a, 1.0 - x, eps) / b


# Cephes `incbet` (Moshier), constants from cephes/cmath/const.c.
MACHEP = 1.11022302462515654042e-16
BIG = 4.503599627370496e15
BIGINV = 2.22044604925031308085e-16
MAXGAM = 171.624376956302725
MAXLOG = 7.09782712893383996843e2
MINLOG = -7.08396418532264106224e2
_MAX_CEPHES_TERMS = 300  # Cephes' own cap
_MAX_SERIES_TERMS = 100_000  # directive 5: pseries converges geometrically for x <= 0.95


def _cephes_fraction(x: float, k: list[float], second: bool) -> float:
    """incbcf (`second` false) or incbd: Cephes' two continued fractions, evaluated forward."""
    k1, k2, k3, k4, k5, k6, k7, k8 = k
    pkm2, qkm2, pkm1, qkm1 = 0.0, 1.0, 1.0, 1.0
    ans, r = 1.0, 1.0
    thresh = 3.0 * MACHEP
    for _ in range(_MAX_CEPHES_TERMS):
        xk = -(x * k1 * k2) / (k3 * k4)
        pk, qk = pkm1 + pkm2 * xk, qkm1 + qkm2 * xk
        pkm2, pkm1, qkm2, qkm1 = pkm1, pk, qkm1, qk
        xk = (x * k5 * k6) / (k7 * k8)
        pk, qk = pkm1 + pkm2 * xk, qkm1 + qkm2 * xk
        pkm2, pkm1, qkm2, qkm1 = pkm1, pk, qkm1, qk
        if qk != 0:
            r = pk / qk
        if r != 0:
            t = abs((ans - r) / r)
            ans = r
        else:
            t = 1.0
        if t < thresh:
            break
        k1 += 1
        k2 += -1 if second else 1
        k3 += 2
        k4 += 2
        k5 += 1
        k6 += 1 if second else -1
        k7 += 2
        k8 += 2
        if abs(qk) + abs(pk) > BIG:
            pkm2, pkm1, qkm2, qkm1 = pkm2 * BIGINV, pkm1 * BIGINV, qkm2 * BIGINV, qkm1 * BIGINV
        if abs(qk) < BIGINV or abs(pk) < BIGINV:
            pkm2, pkm1, qkm2, qkm1 = pkm2 * BIG, pkm1 * BIG, qkm2 * BIG, qkm1 * BIG
    return ans


def cephes_cf(a: float, b: float, x: float) -> float:
    """w of Cephes' incbet: incbcf below x*(a+b-2) = a-1, incbd/(1-x) above."""
    if x * (a + b - 2.0) - (a - 1.0) < 0.0:
        return _cephes_fraction(x, [a, a + b, a, a + 1.0, 1.0, b - 1.0, a + 1.0, a + 2.0], second=False)
    z = x / (1.0 - x)
    return _cephes_fraction(z, [a, b - 1.0, a, a + 1.0, 1.0, a + b, a + 1.0, a + 2.0], second=True) / (
        1.0 - x
    )


Power = Callable[[float, float], float]
Ratio = Callable[[float, float], float]


def cephes_pseries(a: float, b: float, x: float, pw: Power, ratio: Ratio) -> float:
    """Cephes' power series for I_x(a,b), used when b*x <= 1 and x <= 0.95."""
    ai = 1.0 / a
    u = (1.0 - b) * x
    v = u / (a + 1.0)
    t1, t, n, s = v, u, 2.0, 0.0
    z = MACHEP * ai
    while abs(v) > z and n < _MAX_SERIES_TERMS:
        u = (n - b) * x / n
        t *= u
        v = t / (a + n)
        s += v
        n += 1.0
    s += t1
    s += ai
    u = a * math.log(x)
    if a + b < MAXGAM and abs(u) < MAXLOG:
        return s * ratio(a, b) * pw(x, a)
    t = math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + u + math.log(s)
    return 0.0 if t < MINLOG else math.exp(t)


def _cephes_prefactor(a: float, b: float, x: float, xc: float, w: float, *, pw: Power, ratio: Ratio) -> float:
    y, t = a * math.log(x), b * math.log(xc)
    if a + b < MAXGAM and abs(y) < MAXLOG and abs(t) < MAXLOG:
        return pw(xc, b) * pw(x, a) / a * w * ratio(a, b)
    y += t + math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b) + math.log(w / a)
    return 0.0 if y < MINLOG else math.exp(y)


def cephes_incbet(x: float, aa: float, bb: float, pw: Power, ratio: Ratio) -> float:
    """Cephes incbet as published, with the x^a and Gamma-ratio implementations as parameters."""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    if bb * x <= 1.0 and x <= 0.95:
        return cephes_pseries(aa, bb, x, pw, ratio)
    flip = x > aa / (aa + bb)
    a, b, xc, xx = (bb, aa, x, 1.0 - x) if flip else (aa, bb, 1.0 - x, x)
    if flip and b * xx <= 1.0 and xx <= 0.95:
        t = cephes_pseries(a, b, xx, pw, ratio)
    else:
        t = _cephes_prefactor(a, b, xx, xc, cephes_cf(a, b, xx), pw=pw, ratio=ratio)
    if flip:
        return 1.0 - MACHEP if t <= MACHEP else 1.0 - t
    return t


def closed_form(x: float, a: float, b: float) -> float | None:
    """Excel's exact special cases: I_x(a,1) = exp(a ln x), I_x(1,b) = 1 - exp(b ln(1-x))."""
    if x <= 0:
        return 0.0
    if x >= 1:
        return 1.0
    if b == 1.0:
        return exp_log_power(x, a)
    if a == 1.0:
        return 1.0 - math.exp(b * math.log(1.0 - x))
    return None


def best_combination(x: float, a: float, b: float) -> float:
    """Closed forms, else Cephes' fractions only, mean switch, exp-log powers, G correctly rounded."""
    special = closed_form(x, a, b)
    if special is not None:
        return special
    flip = x > a / (a + b)
    if flip:
        x, a, b = 1.0 - x, b, a
    w = cephes_cf(a, b, x)
    if a + b < MAXGAM:
        v = exp_log_power(x, a) * exp_log_power(1.0 - x, b) * gamma_ratio_cr(a, b) / a * w
    else:  # Gamma(a+b) overflows: Cephes' logarithmic path
        v = math.exp(
            a * math.log(x) + b * math.log(1.0 - x) + ln_gamma_cr(a + b) - ln_gamma_cr(a) - ln_gamma_cr(b)
        )
        v = v * w / a
    return 1.0 - v if flip else v


# Derived from LibreOffice sc/source/core/tool/interpr3.cxx (MPL-2.0), ported for fit analysis
# only; engine code must not copy it without the MPL-2.0 notice (deny.toml allows MPL-2.0).
# LibreOffice's GetBetaDist is the closest public relative the
# T-1759 web research found: the same mean reflection, A&S 26.5.8's continued fraction, the
# prefactor pdf*x*y for a, b > 1 with p, q < 0.97 (else exp(a ln x + b ln y - lnB)), and a
# Boost-style Lanczos sum (g = 6.0246800407767296) for Gamma and lnB.
_LO_NUM = [23531376880.41075968857200767445163675473, 42919803642.64909876895789904700198885093,
           35711959237.35566804944018545154716670596, 17921034426.03720969991975575445893111267,
           6039542586.35202800506429164430729792107, 1439720407.311721673663223072794912393972,
           248874557.8620541565114603864132294232163, 31426415.58540019438061423162831820536287,
           2876370.628935372441225409051620849613599, 186056.2653952234950402949897160456992822,
           8071.672002365816210638002902272250613822, 210.8242777515793458725097339207133627117,
           2.506628274631000270164908177133837338626]  # fmt: skip
_LO_DEN = [0.0, 39916800.0, 120543840.0, 150917976.0, 105258076.0, 45995730.0, 13339535.0, 2637558.0,
           357423.0, 32670.0, 1925.0, 66.0, 1.0]  # fmt: skip
_LO_G = 6.024680040776729583740234375
# LibreOffice's own thresholds, kept apart from Cephes' MAXGAM/MAXLOG/MINLOG on purpose: the values
# differ in the last digits and the port must keep LibreOffice's.
_LO_MAX_GAMMA = 171.624376956302
_LO_MAX_ITER = 50000  # LibreOffice's own loop guard
_LOG_DBL_MAX = math.log(sys.float_info.max)
_LOG_DBL_MIN = math.log(sys.float_info.min)


def _lo_lanczos(z: float) -> float:
    if z <= 1.0:
        num, den = _LO_NUM[12], _LO_DEN[12]
        for i in range(11, -1, -1):
            num = num * z + _LO_NUM[i]
            den = den * z + _LO_DEN[i]
    else:
        zi = 1.0 / z
        num, den = _LO_NUM[0], _LO_DEN[0]
        for i in range(1, 13):
            num = num * zi + _LO_NUM[i]
            den = den * zi + _LO_DEN[i]
    return num / den


def _lo_gamma_helper(z: float) -> float:
    g = _lo_lanczos(z)
    zg = z + _LO_G - 0.5
    half = zg ** (z / 2 - 0.25)
    g = g * half / math.exp(zg) * half
    if z <= 20.0 and z == math.floor(z):
        g = float(math.floor(g + 0.5))  # LibreOffice rounds small integer arguments
    return g


def _lo_gamma(z: float) -> float:
    if z >= 1.0:
        return _lo_gamma_helper(z)
    if z >= 0.5:
        return _lo_gamma_helper(z + 1) / z
    return _lo_gamma_helper(z + 2) / (z + 1) / z


def _lo_beta(alpha: float, beta_: float) -> float:
    a, b = (alpha, beta_) if alpha > beta_ else (beta_, alpha)
    if a + b < _LO_MAX_GAMMA:
        return _lo_gamma(a) / _lo_gamma(a + b) * _lo_gamma(b)
    gm = _LO_G - 0.5
    lanczos = _lo_lanczos(a) / _lo_lanczos(a + b) * _lo_lanczos(b)
    lanczos *= math.sqrt(((a + b + gm) / (a + gm)) / (b + gm))
    return math.exp(-a * math.log1p(b / (a + gm)) - b * math.log1p(a / (b + gm)) - gm) * lanczos


def _lo_log_beta(alpha: float, beta_: float) -> float:
    a, b = (alpha, beta_) if alpha > beta_ else (beta_, alpha)
    gm = _LO_G - 0.5
    lanczos = _lo_lanczos(a) / _lo_lanczos(a + b) * _lo_lanczos(b)
    log_lanczos = math.log(lanczos) + 0.5 * (math.log(a + b + gm) - math.log(a + gm) - math.log(b + gm))
    return -a * math.log1p(b / (a + gm)) - b * math.log1p(a / (b + gm)) - gm + log_lanczos


def _lo_pdf(x: float, a: float, b: float) -> float:
    """GetBetaDistPDF for 0 < x < 1 and a, b > 1 (the only branch GetBetaDist calls it on)."""
    log_y = math.log1p(-x) if x < 0.1 else math.log(0.5 - x + 0.5)
    am1, bm1 = (a - 1.0) * math.log(x), (b - 1.0) * log_y
    lb = _lo_log_beta(a, b)
    if all(_LOG_DBL_MIN < v < _LOG_DBL_MAX for v in (am1, bm1, lb, am1 + bm1)):
        return float(x ** (a - 1.0) * (0.5 - x + 0.5) ** (b - 1.0)) / _lo_beta(a, b)
    return math.exp(am1 + bm1 - lb)


def _lo_cont_frac(x: float, a: float, b: float) -> float:
    a1 = b1 = 1.0
    b2 = 1.0 - (a + b) / (a + 1.0) * x
    if b2 == 0.0:
        a2, fnorm, cf = 0.0, 1.0, 1.0
    else:
        a2, fnorm = 1.0, 1.0 / b2
        cf = a2 * fnorm
    cfnew, rm = 1.0, 1.0
    for _ in range(_LO_MAX_ITER):
        apl2m = a + 2.0 * rm
        d2m = rm * (b - rm) * x / ((apl2m - 1.0) * apl2m)
        d2m1 = -(a + rm) * (a + b + rm) * x / (apl2m * (apl2m + 1.0))
        a1 = (a2 + d2m * a1) * fnorm
        b1 = (b2 + d2m * b1) * fnorm
        a2 = a1 + d2m1 * a2 * fnorm
        b2 = b1 + d2m1 * b2 * fnorm
        finished = False
        if b2 != 0.0:
            fnorm = 1.0 / b2
            cfnew = a2 * fnorm
            finished = abs(cf - cfnew) < abs(cf) * sys.float_info.epsilon
        cf = cfnew
        rm += 1.0
        if finished:
            break
    return cf


def libreoffice_beta_dist(x: float, alpha: float, beta_: float, *, excel_closed_forms: bool = False) -> float:
    """LibreOffice GetBetaDist; with `excel_closed_forms`, Excel's b = 1 and a = 1 forms instead of
    LibreOffice's pow(x,a) and -expm1(b*log1p(-x))."""
    if x <= 0.0:
        return 0.0
    if x >= 1.0:
        return 1.0
    if excel_closed_forms:
        special = closed_form(x, alpha, beta_)
        if special is not None:
            return special
    elif beta_ == 1.0:
        return float(x**alpha)
    elif alpha == 1.0:
        return -math.expm1(beta_ * math.log1p(-x))
    y, ln_y, ln_x, a, b, xx = (0.5 - x) + 0.5, math.log1p(-x), math.log(x), alpha, beta_, x
    reflect = x > alpha / (alpha + beta_)
    if reflect:
        a, b, xx, y, ln_x, ln_y = beta_, alpha, y, x, ln_y, math.log(x)
    result = _lo_cont_frac(xx, a, b) / a
    p, q = a / (a + b), b / (a + b)
    if a > 1.0 and b > 1.0 and p < 0.97 and q < 0.97:  # LibreOffice: "found experimental"
        result *= _lo_pdf(xx, a, b) * xx * y
    else:
        result *= math.exp(a * ln_x + b * ln_y - _lo_log_beta(a, b))
    if reflect:
        result = 0.5 - result + 0.5
    return min(max(result, 0.0), 1.0)


def ibeta_candidates() -> list[tuple[str, str, Ibeta]]:
    """(name, family, I_x(a,b)) for every candidate; the fits map their arguments onto these."""
    out: list[tuple[str, str, Ibeta]] = [("correctly rounded", "reference", ibeta_reference)]
    for log1p_tail in (False, True):
        for eps in (3e-16, 1e-15, 1e-14):
            name = f"NR betai eps={eps:g}, ln(1-x) by {'log1p' if log1p_tail else 'log'}"

            def nr(x: float, a: float, b: float, tail: bool = log1p_tail, e: float = eps) -> float:
                return nr_betai(x, a, b, tail, e)

            out.append((name, "NR", nr))
    for pname, pw in (("pow", libm_pow), ("exp(a ln x)", exp_log_power)):
        for gname, ratio in (("tgamma", gamma_ratio_libm), ("Gamma correctly rounded", gamma_ratio_cr)):

            def cephes(x: float, a: float, b: float, p: Power = pw, r: Ratio = ratio) -> float:
                return cephes_incbet(x, a, b, p, r)

            out.append((f"Cephes incbet, x^a by {pname}, {gname}", "Cephes", cephes))

    def cephes_closed(x: float, a: float, b: float) -> float:
        special = closed_form(x, a, b)
        return special if special is not None else cephes_incbet(x, a, b, exp_log_power, gamma_ratio_cr)

    out.append(("Cephes incbet + closed forms", "Cephes", cephes_closed))
    out.append(("closed forms, Cephes fractions, mean switch (best)", "best", best_combination))
    out.append(("LibreOffice GetBetaDist", "LibreOffice", libreoffice_beta_dist))

    def libreoffice_closed(x: float, a: float, b: float) -> float:
        return libreoffice_beta_dist(x, a, b, excel_closed_forms=True)

    out.append(("LibreOffice GetBetaDist + Excel closed forms", "LibreOffice", libreoffice_closed))
    return out


def _numbers(args: tuple[Arg, ...]) -> list[float]:
    return [numeric_arg(a) for a in args]


def beta_dist_candidates() -> list[Candidate]:
    def wrap(fn: Ibeta) -> Callable[..., float]:
        def call(*args: Arg) -> float:
            x, a, b = _numbers(args)[:3]
            return fn(x, a, b)

        return call

    return [candidate(name, wrap(fn), family) for name, family, fn in ibeta_candidates()]


def f_cdf(fn: Ibeta, x: float, d1: float, d2: float) -> float:
    """F.DIST(x,d1,d2,TRUE) = I_y(d1/2, d2/2), y = d1*x/(d1*x + d2). Excel truncates both df
    (t1049-fdist: F.DIST(1,3.9,7.9,TRUE) = F.DIST(1,3,7,TRUE))."""
    d1, d2 = float(math.trunc(d1)), float(math.trunc(d2))
    return fn(d1 * x / (d1 * x + d2), d1 / 2.0, d2 / 2.0)


def f_cdf_rt(fn: Ibeta, x: float, d1: float, d2: float) -> float:
    """F.DIST.RT(x,d1,d2) = I_w(d2/2, d1/2), w = d2/(d2 + d1*x), df truncated as in `f_cdf`."""
    d1, d2 = float(math.trunc(d1)), float(math.trunc(d2))
    return fn(d2 / (d2 + d1 * x), d2 / 2.0, d1 / 2.0)


def t_two_tail(fn: Ibeta, t: float, n: float) -> float:
    """P(|T| > |t|) = I_y(n/2, 1/2), y = n/(n + t^2); n truncated (statistics.md, T-1043)."""
    n = float(math.trunc(n))
    return fn(n / (n + t * t), n / 2.0, 0.5)


def t_cdf(fn: Ibeta, t: float, n: float) -> float:
    """T.DIST(t,n,TRUE) = 1 - I_y(n/2, 1/2)/2 for t > 0 (I_y/2 below), y = n/(n + t^2), or
    0.5 +- I_z(1/2, n/2)/2 with z = 1 - y when z is smaller (back to I_y/2 for a lower tail under
    0.1, where the difference cancels); Excel truncates n to an integer
    (docs/excel-semantics/statistics.md, T-1043)."""
    n = float(math.trunc(n))
    if t == 0:
        return 0.5
    # Map through the smaller of y = n/(n+t^2) and z = t^2/(n+t^2): rounding y near 1 before I_y
    # loses the tail (about 7E5 ulp at n = 1E6).
    y, z = n / (n + t * t), t * t / (n + t * t)
    if y <= z:
        half_tail = fn(y, n / 2.0, 0.5) / 2.0
        return 1.0 - half_tail if t > 0 else half_tail
    centre = fn(z, 0.5, n / 2.0) / 2.0
    if t > 0:
        return 0.5 + centre
    lower = 0.5 - centre
    # 0.5 - centre cancels in the deep lower tail (T.DIST(-10,1000,TRUE) = 8.3E-23): take I_y/2.
    return lower if lower >= 0.1 else fn(y, n / 2.0, 0.5) / 2.0


def t_cdf_reference(t: float, n: float) -> float:
    """T.DIST(t,n,TRUE) correctly rounded: y = n/(n + t^2) stays an exact decimal into I_y, because
    rounding y to a double first moves the deep tail (n = 1E6, t = -2) by 152,191 ulp (T-2088)."""
    n = float(math.trunc(n))
    if t == 0:
        return 0.5
    dn, dt = decimal.Decimal(n), decimal.Decimal(t)
    y = _CTX.divide(dn, _CTX.add(dn, _CTX.multiply(dt, dt)))
    half_tail = _ibeta_decimal(y, dn / 2, decimal.Decimal("0.5")) / 2
    return float(1 - half_tail if t > 0 else half_tail)


def f_dist_candidates() -> list[Candidate]:
    """F.DIST(x,d1,d2,TRUE) through `f_cdf` for every I_x candidate."""

    def wrap(fn: Ibeta) -> Callable[..., float]:
        def call(*args: Arg) -> float:
            x, d1, d2 = _numbers(args)
            return f_cdf(fn, x, d1, d2)

        return call

    return [candidate(name, wrap(fn), family) for name, family, fn in ibeta_candidates()]


def t_dist_candidates() -> list[Candidate]:
    """T.DIST(t,n,TRUE) through `t_cdf` for every I_x candidate."""

    def wrap(fn: Ibeta) -> Callable[..., float]:
        def call(*args: Arg) -> float:
            t, n = _numbers(args)
            return t_cdf(fn, t, n)

        return call

    def reference(*args: Arg) -> float:
        t, n = _numbers(args)
        return t_cdf_reference(t, n)

    # The reference maps exactly, not through a double y (see `t_cdf_reference`).
    return [candidate("correctly rounded", reference, "reference")] + [
        candidate(name, wrap(fn), family) for name, family, fn in ibeta_candidates() if family != "reference"
    ]


FITS = [
    Fit(
        name="beta-dist",
        description="BETA.DIST(x,a,b,TRUE)/BETADIST (T-1044, T-1047): I_x(a,b), NR, Cephes, closed forms",
        cases=_BETA_CASES,
        pattern=_BETA_PATTERN,
        candidates=beta_dist_candidates,
    ),
    Fit(
        name="f-dist",
        description="F.DIST(x,d1,d2,TRUE) (T-1044, T-1049): the beta-dist candidates at y = d1x/(d1x+d2)",
        cases=_F_CASES,
        pattern=_F_PATTERN,
        candidates=f_dist_candidates,
    ),
    Fit(
        name="t-dist",
        description="T.DIST(t,n,TRUE) (T-1043, T-1048): the beta-dist candidates at y = n/(n+t^2)",
        cases=_T_CASES,
        pattern=_T_PATTERN,
        candidates=t_dist_candidates,
    ),
]
