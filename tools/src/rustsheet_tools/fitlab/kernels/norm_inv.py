"""Inverse standard normal: NORM.S.INV, NORMSINV and NORM.INV(p,0,1) (T-1746, rank 11 in
docs/research/numeric-kernel-map.md).

One fit, `norm-s-inv`, over the T-1006, T-1048 and T-1746 sweeps (probes-numeric/m9-norm-s-inv,
t1048-normsinv, t1048-norminv, t1746-norm-s-inv-central). Most m9-norm-s-inv cells take p = NORM.S.DIST(k,TRUE), a nested
call the sweep reader does not evaluate, so the pattern captures k and a candidate looks p up in
Excel's own NORM.S.DIST(k,TRUE) values (the T-1040/T-1048/M9 NORM.S.DIST sweeps cover every k the
inverse sweep uses); a literal p is used as is.

Candidates, each plain IEEE double (Python floats, platform libm for log/sqrt/exp/erfc):
- Wichura's AS 241: PPND16 (the double-precision coefficients) and PPND7, with the tail argument
  r = sqrt(-ln(min(p, 1-p))) taken as `log(p)`/`log(1-p)` or `log1p(-p)` on the upper side, and
  the rational functions by Horner (as published) or as ascending power sums;
- PPND16 plus one Newton or Halley step against libm's erfc, and one Newton step against
  0.5*erfc(-x*sqrt(0.5)), the CDF form closest to Excel's NORM.S.DIST on the T-1746 probe (the
  "iterate on Excel's own CDF" hypothesis that T-0925's notes raise);
- statrs' `-sqrt(2)*erfc_inv(2p)` (Boost's erf_inv regions; the engine today), and the hybrid
  PPND16 in the tails with statrs centrally (|p - 0.5| <= 0.425);
- Acklam's rational approximation, raw and with its published Halley refinement;
- Moro's (Beasley-Springer-Moro) algorithm;
- the correctly rounded inverse of p, from a 60-digit Newton solve (decimal), as a reference.

Result (T-1746 Outcome; docs/excel-semantics/statistics.md, "NORM.S.INV is AS 241 PPND16 in the
tails"): PPND16 by Horner, upper tail via log(1-p), is bit-exact on 611 of 616 tail cells
(|p - 0.5| > 0.425), and none of the candidates fits the central region, where Excel is up to 31 ulp from the correctly
rounded value, symmetric in p and 1 - p, and does not round-trip through its own NORM.S.DIST.
Cells whose p reaches the kernel through Excel's input handling (text p, a 16-digit literal Excel
reads to 15 digits, a subnormal operand Excel flushes) are left out by `kernel_input`.
"""

from __future__ import annotations

import decimal
import math
from collections.abc import Callable, Sequence
from functools import cache

from rustsheet_tools.fitlab.fit import Fit
from rustsheet_tools.fitlab.score import Candidate, candidate
from rustsheet_tools.fitlab.sweep import Arg, Cell, load_sweep, numeric_arg

CASES = [
    "probes-numeric/m9-norm-s-inv",
    "probes-numeric/t1048-normsinv",
    "probes-numeric/t1048-norminv",
    "probes-numeric/t1746-norm-s-inv-central",
]
# Keyed `<case file> <Sheet!A1>` as `Cell.case` and `Cell.key` give them. Cells whose p is shaped by Excel's input handling before the kernel sees it: both are PPND16
# bit-exact at the p Excel actually uses (0.999999999999999, 2.3E-308).
_INPUT_QUIRKS = {
    "probes-numeric/t1048-normsinv.case.json Calc!A21",  # =NORMSINV(0.9999999999999999): 15-digit literal
    "probes-numeric/t1048-normsinv.case.json Calc!B21",  # =NORM.S.INV(0.9999999999999999)
    "probes-numeric/m9-norm-s-inv.case.json Calc!A648",  # =NORM.S.INV(2.3E-308-1E-309): 1E-309 flushes to 0
}
# Group 1: k of a nested NORM.S.DIST(k,TRUE), with group 2 its TRUE (so a candidate sees two
# arguments); group 3: a literal p (one argument). NORM.INV only with mean 0 and sd 1.
PATTERN = (
    r"^=(?:NORM\.S\.INV|NORMSINV|NORM\.INV|NORMINV)\("
    r"(?:NORM\.S\.DIST\(([^(),]+),(TRUE)\)|([^(),]+))(?:,0,1)?\)$"
)
# Excel's NORM.S.DIST(k,TRUE) (and NORMSDIST(k)) per k, for the nested cells.
_CDF_CASES = [
    "probes-numeric/t1040-norm-s-dist-cdf-a",
    "probes-numeric/t1040-norm-s-dist-cdf-b",
    "probes-numeric/m9-normpdf-norm-s-dist",
    "probes-numeric/t1048-normsdist",
    "probes-numeric/m9-norm-s-inv",
]
_CDF_PATTERN = r"^=(?:NORM\.S\.DIST\(([^(),]+),TRUE\)|NORMSDIST\(([^(),]+)\))$"

Poly = Sequence[float]


@cache
def excel_cdf() -> dict[float, float]:
    """Excel's NORM.S.DIST(k,TRUE) per literal k, from the oracle-filled sweeps."""
    sweep = load_sweep(_CDF_CASES, _CDF_PATTERN, None, False)
    # Text and bool k (coercion probes) are not inverse-sweep inputs.
    return {k: c.expected for c in sweep.cells if isinstance(k := c.args[0], float) and not c.error}


def p_of(args: tuple[Arg, ...]) -> float:
    """The probability a cell passes: Excel's NORM.S.DIST(k) for a nested cell, else the literal."""
    if len(args) == 2:  # (k, TRUE): nested NORM.S.DIST
        k = numeric_arg(args[0])
        table = excel_cdf()
        if k not in table:
            msg = f"no Excel NORM.S.DIST({k},TRUE) in the CDF sweeps"
            raise KeyError(msg)
        return table[k]
    return numeric_arg(args[0])


def kernel_input(cell: Cell) -> bool:
    """Leaves out text p (a coercion probe) and the `_INPUT_QUIRKS` cells."""
    return not any(isinstance(a, str) for a in cell.args) and f"{cell.case} {cell.key}" not in _INPUT_QUIRKS


def _domain(p: float) -> None:
    if not 0.0 < p < 1.0:
        msg = f"p = {p} is outside (0, 1)"
        raise ValueError(msg)


def horner(coeffs: Poly, x: float) -> float:
    """c[0] + c[1] x + ... by Horner from the top coefficient, as AS 241 publishes it."""
    acc = coeffs[-1]
    for c in reversed(coeffs[:-1]):
        acc = acc * x + c
    return acc


def power_sum(coeffs: Poly, x: float) -> float:
    """c[0] + c[1] x + ... summed upward with explicit powers (the 'reversed' evaluation order)."""
    acc, xp = 0.0, 1.0
    for c in coeffs:
        acc += c * xp
        xp *= x
    return acc


Evaluate = Callable[[Poly, float], float]

# Wichura, Algorithm AS 241 (Applied Statistics 37, 1988): PPND16, ascending coefficients.
PPND16 = {
    "a": [
        3.3871328727963666080e0,
        1.3314166789178437745e2,
        1.9715909503065514427e3,
        1.3731693765509461125e4,
        4.5921953931549871457e4,
        6.7265770927008700853e4,
        3.3430575583588128105e4,
        2.5090809287301226727e3,
    ],
    "b": [
        1.0,
        4.2313330701600911252e1,
        6.8718700749205790830e2,
        5.3941960214247511077e3,
        2.1213794301586595867e4,
        3.9307895800092710610e4,
        2.8729085735721942674e4,
        5.2264952788528545610e3,
    ],
    "c": [
        1.42343711074968357734e0,
        4.63033784615654529590e0,
        5.76949722146069140550e0,
        3.64784832476320460504e0,
        1.27045825245236838258e0,
        2.41780725177450611770e-1,
        2.27238449892691845833e-2,
        7.74545014278341407640e-4,
    ],
    "d": [
        1.0,
        2.05319162663775882187e0,
        1.67638483018380384940e0,
        6.89767334985100004550e-1,
        1.48103976427480074590e-1,
        1.51986665636164571966e-2,
        5.47593808499534494600e-4,
        1.05075007164441684324e-9,
    ],
    "e": [
        6.65790464350110377720e0,
        5.46378491116411436990e0,
        1.78482653991729133580e0,
        2.96560571828504891230e-1,
        2.65321895265761230930e-2,
        1.24266094738807843860e-3,
        2.71155556874348757815e-5,
        2.01033439929228813265e-7,
    ],
    "f": [
        1.0,
        5.99832206555887937690e-1,
        1.36929880922735805310e-1,
        1.48753612908506148525e-2,
        7.86869131145613259100e-4,
        1.84631831751005468180e-5,
        1.42151175831644588870e-7,
        2.04426310338993978564e-15,
    ],
}
PPND7 = {
    "a": [3.3871327179, 50.434271938, 159.29113202, 59.109374720],
    "b": [1.0, 17.895169469, 78.757757664, 67.187563600],
    "c": [1.4234372777, 2.7568153900, 1.3067284816, 0.17023821103],
    "d": [1.0, 0.73700164250, 0.12021132975],
    "e": [6.6579051150, 3.0812263860, 0.42868294337, 0.017337203997],
    "f": [1.0, 0.24197894225, 0.012258202635],
}
SPLIT1, SPLIT2, CONST1, CONST2 = 0.425, 5.0, 0.180625, 1.6


def as241(p: float, coeffs: dict[str, list[float]], ev: Evaluate, log1p_upper: bool) -> float:
    """AS 241 (PPND16/PPND7 by `coeffs`): a rational in q^2 centrally, in sqrt(-ln r) in the tails."""
    _domain(p)
    q = p - 0.5
    if abs(q) <= SPLIT1:
        r = CONST1 - q * q
        return q * ev(coeffs["a"], r) / ev(coeffs["b"], r)
    if q < 0:
        r = math.sqrt(-math.log(p))
    else:
        r = math.sqrt(-(math.log1p(-p) if log1p_upper else math.log(1.0 - p)))
    if r <= SPLIT2:
        r -= CONST2
        val = ev(coeffs["c"], r) / ev(coeffs["d"], r)
    else:
        r -= SPLIT2
        val = ev(coeffs["e"], r) / ev(coeffs["f"], r)
    return -val if q < 0 else val


SQRT2 = math.sqrt(2.0)
SQRT2PI = math.sqrt(2.0 * math.pi)
SQRT_HALF = math.sqrt(0.5)


def libm_cdf(x: float) -> float:
    return 0.5 * math.erfc(-x / SQRT2)


def sqrt_half_cdf(x: float) -> float:
    """0.5*erfc(-x*sqrt(0.5)): equal to Excel's NORM.S.DIST on 260 of the 341 T-1746 probe points
    (0.5*erfc(-x/sqrt(2)) on 199), so the closest stand-in for Excel's own CDF the lab has."""
    return 0.5 * math.erfc(-x * SQRT_HALF)


def refine(p: float, x: float, halley: bool, cdf: Callable[[float], float] = libm_cdf) -> float:
    """One Newton (or Halley) step on cdf(x) = p."""
    e = cdf(x) - p
    u = e * SQRT2PI * math.exp(x * x / 2.0)
    return x - u / (1.0 + x * u / 2.0) if halley else x - u


ACKLAM_A = [-3.969683028665376e01, 2.209460984245205e02, -2.759285104469687e02, 1.383577518672690e02,
            -3.066479806614716e01, 2.506628277459239e00]  # fmt: skip
ACKLAM_B = [-5.447609879822406e01, 1.615858368580409e02, -1.556989798598866e02, 6.680131188771972e01,
            -1.328068155288572e01]  # fmt: skip
ACKLAM_C = [-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e00, -2.549732539343734e00,
            4.374664141464968e00, 2.938163982698783e00]  # fmt: skip
ACKLAM_D = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e00, 3.754408661907416e00]
ACKLAM_LOW = 0.02425


def _descending(coeffs: Poly, x: float) -> float:
    acc = 0.0
    for c in coeffs:
        acc = acc * x + c
    return acc


def acklam(p: float, halley: bool) -> float:
    """Acklam's rational approximation (relative error 1.15e-9), optionally Halley-refined."""
    _domain(p)
    if p < ACKLAM_LOW:
        q = math.sqrt(-2.0 * math.log(p))
        x = _descending(ACKLAM_C, q) / (_descending(ACKLAM_D, q) * q + 1.0)
    elif p <= 1.0 - ACKLAM_LOW:
        q = p - 0.5
        r = q * q
        x = _descending(ACKLAM_A, r) * q / (_descending(ACKLAM_B, r) * r + 1.0)
    else:
        q = math.sqrt(-2.0 * math.log(1.0 - p))
        x = -_descending(ACKLAM_C, q) / (_descending(ACKLAM_D, q) * q + 1.0)
    return refine(p, x, halley=True) if halley else x


MORO_A = [2.50662823884, -18.61500062529, 41.39119773534, -25.44106049637]
MORO_B = [-8.47351093090, 23.08336743743, -21.06224101826, 3.13082909833]
MORO_C = [0.3374754822726147, 0.9761690190917186, 0.1607979714918209, 0.0276438810333863,
          0.0038405729373609, 0.0003951896511919, 0.0000321767881768, 0.0000002888167364,
          0.0000003960315187]  # fmt: skip


def moro(p: float) -> float:
    """Beasley-Springer-Moro: Beasley-Springer centrally, Moro's Chebyshev fit in the tails."""
    _domain(p)
    y = p - 0.5
    if abs(y) < 0.42:
        r = y * y
        return y * horner(MORO_A, r) / (horner([1.0, *MORO_B], r))
    r = p if y < 0 else 1.0 - p
    s = math.log(-math.log(r))
    x = horner(MORO_C, s)
    return -x if y < 0 else x


# statrs 0.18 `erf_inv_impl` (Boost's erf_inv), the three regions the central p reach.
STATRS_AN = [-0.000508781949658280665617, -0.00836874819741736770379, 0.0334806625409744615033,
             -0.0126926147662974029034, -0.0365637971411762664006, 0.0219878681111168899165,
             0.00822687874676915743155, -0.00538772965071242932965]  # fmt: skip
STATRS_AD = [1.0, -0.970005043303290640362, -1.56574558234175846809, 1.56221558398423026363,
             0.662328840472002992063, -0.71228902341542847553, -0.0527396382340099713954,
             0.0795283687341571680018, -0.00233393759374190016776, 0.000886216390456424707504]  # fmt: skip
STATRS_BN = [-0.202433508355938759655, 0.105264680699391713268, 8.37050328343119927838,
             17.6447298408374015486, -18.8510648058714251895, -44.6382324441786960818,
             17.445385985570866523, 21.1294655448340526258, -3.67192254707729348546]  # fmt: skip
STATRS_BD = [1.0, 6.24264124854247537712, 3.9713437953343869095, -28.6608180499800029974,
             -20.1432634680485188801, 48.5609213108739935468, 10.8268667355460159008,
             -22.6436933413139721736, 1.72114765761200282724]  # fmt: skip
STATRS_CN = [-0.131102781679951906451, -0.163794047193317060787, 0.117030156341995252019,
             0.387079738972604337464, 0.337785538912035898924, 0.142869534408157156766,
             0.0290157910005329060432, 0.00214558995388805277169, -0.679465575181126350155e-6,
             0.285225331782217055858e-7, -0.681149956853776992068e-9]  # fmt: skip
STATRS_CD = [1.0, 3.46625407242567245975, 5.38168345707006855425, 4.77846592945843778382,
             2.59301921623620271374, 0.848854343457902036425, 0.152264338295331783612,
             0.01105924229346489121]  # fmt: skip


def _statrs_erf_inv_impl(p: float, q: float, sign: float) -> float:
    if p <= 0.5:
        g = p * (p + 10.0)
        r = horner(STATRS_AN, p) / horner(STATRS_AD, p)
        return sign * (g * 0.0891314744949340820313 + g * r)
    if q >= 0.25:
        g = math.sqrt(-2.0 * math.log(q))
        xs = q - 0.25
        r = horner(STATRS_BN, xs) / horner(STATRS_BD, xs)
        return sign * (g / (2.249481201171875 + r))
    x = math.sqrt(-math.log(q))
    if x >= 3.0:
        msg = "statrs region beyond the central port"
        raise ValueError(msg)
    xs = x - 1.125
    r = horner(STATRS_CN, xs) / horner(STATRS_CD, xs)
    return sign * (0.807220458984375 * x + r * x)


def statrs_central(p: float) -> float:
    """statrs `Normal::inverse_cdf`: -(sqrt(2) * erfc_inv(2p)), for p in the AS 241 central region."""
    _domain(p)
    x = 2.0 * p
    inv = _statrs_erf_inv_impl(-1.0 + x, 2.0 - x, -1.0) if x > 1.0 else _statrs_erf_inv_impl(1.0 - x, x, 1.0)
    return -(SQRT2 * inv)


def hybrid(p: float) -> float:
    """PPND16 in the tails (where it is bit-exact), statrs centrally (its best candidate)."""
    if abs(p - 0.5) <= SPLIT1:
        return statrs_central(p)
    return as241(p, PPND16, horner, log1p_upper=False)


# A 60-digit reference: Phi by erf's Taylor series (|z| < 3) or erfc's continued fraction.
_DIGITS = 60
_CTX = decimal.Context(prec=_DIGITS + 20)
_PI = decimal.Decimal("3.14159265358979323846264338327950288419716939937510582097494459230781640628620899")
_MAX_TERMS = 2000  # bounded loops (directive 5): both expansions converge far sooner for |z| <= 30


def _erf_series(z: decimal.Decimal) -> decimal.Decimal:
    c = _CTX
    total, term, n = z, z, 0
    z2 = c.multiply(z, z)
    while n < _MAX_TERMS:
        n += 1
        term = c.divide(c.multiply(-term, z2), n)
        piece = c.divide(term, 2 * n + 1)
        total = c.add(total, piece)
        if abs(piece) < abs(total) * decimal.Decimal(10) ** -(_DIGITS + 10):
            break
    return c.multiply(total, c.divide(2, c.sqrt(_PI)))


def _erfc_fraction(z: decimal.Decimal) -> decimal.Decimal:
    """erfc(z), z >= 3: exp(-z^2)/sqrt(pi) / (z + (1/2)/(z + 1/(z + (3/2)/(z + ...)))), bottom-up."""
    c = _CTX
    t = z
    for k in range(_MAX_TERMS, 0, -1):
        t = c.add(z, c.divide(decimal.Decimal(k) / 2, t))
    return c.divide(c.exp(-c.multiply(z, z)), c.multiply(c.sqrt(_PI), t))


def _erfc(z: decimal.Decimal) -> decimal.Decimal:
    if z >= 3:
        return _erfc_fraction(z)
    if z <= -3:
        return 2 - _erfc_fraction(-z)
    return 1 - _erf_series(z)


def _phi(x: decimal.Decimal) -> decimal.Decimal:
    return _erfc(-x / _CTX.sqrt(decimal.Decimal(2))) / 2


def correctly_rounded(p: float) -> float:
    """Phi^-1(p) rounded once to a double: Newton at 80 digits from the PPND16 value."""
    _domain(p)
    c = _CTX
    target = decimal.Decimal(p)
    x = decimal.Decimal(as241(p, PPND16, horner, log1p_upper=False))
    root2pi = c.sqrt(2 * _PI)
    for _ in range(50):
        density = c.divide(c.exp(-c.multiply(x, x) / 2), root2pi)
        step = c.divide(_phi(x) - target, density)
        x = c.subtract(x, step)
        if abs(step) <= abs(x) * decimal.Decimal(10) ** -(_DIGITS - 5):
            break
    return float(x)


def _p(fn: Callable[[float], float]) -> Callable[..., float]:
    def call(*args: Arg) -> float:
        return fn(p_of(args))

    return call


def norm_inv_candidates() -> list[Candidate]:
    out: list[Candidate] = [candidate("correctly rounded", _p(correctly_rounded), "reference")]
    for cname, coeffs in (("PPND16", PPND16), ("PPND7", PPND7)):
        for ename, ev in (("horner", horner), ("power sum", power_sum)):
            for log1p_upper in (False, True):
                label = f"{cname} {ename}, upper {'log1p(-p)' if log1p_upper else 'log(1-p)'}"

                def fn(
                    p: float, co: dict[str, list[float]] = coeffs, e: Evaluate = ev, lu: bool = log1p_upper
                ) -> float:
                    return as241(p, co, e, lu)

                out.append(candidate(label, _p(fn), cname))
    for halley in (False, True):
        step = "Halley" if halley else "Newton"

        def stepped(p: float, h: bool = halley) -> float:
            return refine(p, as241(p, PPND16, horner, log1p_upper=False), h)

        out.append(candidate(f"PPND16 + one {step} step (libm erfc)", _p(stepped), "PPND16+step"))
    out.append(
        candidate(
            "PPND16 + one Newton step (0.5*erfc(-x*sqrt(0.5)))",
            _p(lambda p: refine(p, as241(p, PPND16, horner, log1p_upper=False), False, sqrt_half_cdf)),
            "PPND16+step",
        )
    )
    out.append(candidate("PPND16 tails, statrs centre", _p(hybrid), "hybrid"))
    out.append(
        candidate(
            "statrs centre only",
            _p(lambda p: statrs_central(p) if abs(p - 0.5) <= SPLIT1 else math.nan),
            "statrs",
        )
    )
    out.append(candidate("Acklam", _p(lambda p: acklam(p, halley=False)), "Acklam"))
    out.append(candidate("Acklam + Halley (libm erfc)", _p(lambda p: acklam(p, halley=True)), "Acklam"))
    out.append(candidate("Moro", _p(moro), "Moro"))
    return out


FITS = [
    Fit(
        name="norm-s-inv",
        description="NORM.S.INV/NORMSINV/NORM.INV(p,0,1) (T-1006, T-1048): AS 241, Acklam, Moro",
        cases=CASES,
        pattern=PATTERN,
        candidates=norm_inv_candidates,
        where=kernel_input,
        notes=[f"nested p = Excel's NORM.S.DIST(k,TRUE) from {', '.join(_CDF_CASES)}"],
    ),
]
