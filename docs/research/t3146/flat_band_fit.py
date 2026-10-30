"""T-3146: score search forms for Excel's gamma quantiles near p = 0.999 against the T-3148 probe.

Cells: every GAMMA.INV and CHISQ.INV.RT cell of probes-numeric/t3148-gamma-quantile-flat-band (140), with the formula
literals cut to 15 significant digits as Excel reads them. The scored set "S" leaves out shapes 0.5 and 1 (CHISQ.INV.RT
at df 1 and 2, GAMMA.INV at shape 1 are closed forms): 112 cells. "S99" is S at 0.99 <= p <= 0.9999 (A120-A140 count
here, their p read as 0.998999999999999 and 0.999).

"Our double cdf" is the correctly rounded P (GAMMA.INV) or Q (CHISQ.INV.RT, of x / 2 at shape df / 2), from mpmath at
60 digits. The engine's accurate pair is within an ulp of it; the engine is not run here. Each search runs on that
double cdf, in unit scale (CHISQ.INV.RT doubles the result). Scores: bit-exact / within 4 ulp / visible (within half a
unit of Excel's 15th significant digit).

Usage (repo root): uv run --no-project --with mpmath python docs/research/t3146/flat_band_fit.py
Output: docs/research/t3146/flat_band_fit.out
"""
import json
import math
import re
from decimal import ROUND_DOWN, Decimal

import mpmath as mp

mp.mp.dps = 60
CASE = "eval/suites/probes-numeric/cases/t3148-gamma-quantile-flat-band"


def cut15(text):
    d = Decimal(text)
    return float(d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN))


def load():
    cells = json.load(open(CASE + ".case.json"))["sheets"][0]["cells"]
    oracle = json.load(open(CASE + ".oracle.json"))["sheets"]["Calc"]
    out = []
    for key, formula in cells.items():
        m = re.fullmatch(r"=(GAMMA\.INV|CHISQ\.INV\.RT)\(([^,]+),([^,)]+)(?:,([^)]+))?\)", formula)
        if not m:
            continue
        p = cut15(m[2])
        if m[1] == "GAMMA.INV":
            a, scale, upper = cut15(m[3]), cut15(m[4]), False
        else:
            a, scale, upper = int(float(m[3])) / 2, 2.0, True
        out.append(dict(cell=key, formula=formula, p=p, a=a, scale=scale, upper=upper, excel=oracle[key]["v"]))
    return out


def tail(a, x, upper):
    """The double cdf of the given tail at unit-scale x: correctly rounded."""
    if x <= 0:
        return 1.0 if upper else 0.0
    if upper:
        return float(mp.gammainc(a, x, mp.inf, regularized=True))
    return float(mp.gammainc(a, 0, x, regularized=True))


def density(a, x):
    return float(mp.exp((a - 1) * mp.log(x) - x - mp.loggamma(a)))


def crossing(r):
    a, p, up = mp.mpf(r["a"]), mp.mpf(r["p"]), r["upper"]
    g = (lambda x: mp.gammainc(a, x, mp.inf, regularized=True) - p) if up else (
        lambda x: mp.gammainc(a, 0, x, regularized=True) - p)
    return float(mp.findroot(g, mp.mpf(r["excel"] / r["scale"]), solver="secant", tol=mp.mpf(10) ** -45))


def newton(r, x0, mode):
    """mode 0: return x where the double cdf equals p; mode 1: return x_{n+1} once the step is under 1E-15 of x."""
    a, p, up = r["a"], r["p"], r["upper"]
    x = x0
    for _ in range(200):
        v = tail(a, x, up)
        if mode == 0 and v == p:
            return x
        d = density(a, x)
        step = ((v - p) / d) * (-1 if up else 1)
        nx = x - step
        if nx <= 0:
            nx = x / 2
        if mode == 1 and abs(nx - x) <= 1e-15 * abs(x):
            return nx
        if nx == x:
            return x
        x = nx
    return math.nan


def newton_log_cdf(r, x0):
    a, p, up = r["a"], r["p"], r["upper"]
    x = x0
    for _ in range(200):
        v = tail(a, x, up)
        if v == p:
            return x
        step = (math.log(v) - math.log(p)) / (density(a, x) / v) * (-1 if up else 1)
        nx = x - step
        if nx <= 0:
            nx = x / 2
        if nx == x:
            return x
        x = nx
    return math.nan


def newton_log_x(r, x0):
    a, p, up = r["a"], r["p"], r["upper"]
    t = math.log(x0)
    for _ in range(200):
        x = math.exp(t)
        v = tail(a, x, up)
        if v == p:
            return x
        nt = t - (v - p) / (density(a, x) * x) * (-1 if up else 1)
        if nt == t:
            return x
        t = nt
    return math.nan


def bisect(r):
    a, p, up = r["a"], r["p"], r["upper"]

    def below(x):
        v = tail(a, x, up)
        return v > p if up else v < p

    lo, hi = 0.0, 1.0
    while below(hi):
        lo, hi = hi, hi * 2
    for _ in range(200):
        m = 0.5 * (lo + hi)
        if tail(a, m, up) == p:
            return m
        if below(m):
            lo = m
        else:
            hi = m
        if m in (lo, hi) and lo == hi:
            break
    return 0.5 * (lo + hi)


def statrs_search(r):
    """statrs 0.18 Gamma::inverse_cdf on the lower tail: halve/double a bracket, 8 bisections, 4 Newton steps
    (relative convergence 1E-10)."""
    a, up = r["a"], r["upper"]
    p = 1 - r["p"] if up else r["p"]
    lo, hi = 1.0, 2.0
    while tail(a, lo, False) > p:
        lo /= 2
    while tail(a, hi, False) < p:
        hi *= 2
    x = (hi + lo) / 2

    def conv(old, new):
        return abs(old - new) <= max(1e-10 * max(abs(old), abs(new)), 2.220446049250313e-16)

    for _ in range(8):
        if tail(a, x, False) >= p:
            hi = x
        else:
            lo = x
        nx = (hi + lo) / 2
        done = conv(x, nx)
        x = nx
        if done:
            break
    for _ in range(4):
        nx = x - (tail(a, x, False) - p) / density(a, x)
        done = conv(x, nx)
        x = nx
        if done:
            break
    return x


def wilson_hilferty(r):
    a, p = r["a"], r["p"]
    lower = 1 - p if r["upper"] else p
    z = float(mp.sqrt(2) * mp.erfinv(2 * mp.mpf(lower) - 1))
    c = 1 - 1 / (9 * a) + z / (3 * math.sqrt(a))
    # A negative cube (small shape, far lower tail) starts at a / 1000 instead.
    return a * c ** 3 if c > 0 else a / 1000


def vis(x, e):
    return abs(x - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14)


def main():
    rows = load()
    s112 = [r for r in rows if r["a"] not in (0.5, 1.0)]
    s99 = [r for r in s112 if 0.99 <= r["p"] <= 0.9999]
    print(f"quantile cells {len(rows)}; S (shape not 0.5 or 1) {len(s112)}; S99 (and 0.99 <= p <= 0.9999) {len(s99)}")
    # Where Excel's quantile sits on our double cdf.
    for name, rs in (("all 140", rows), ("S", s112), ("S99", s99)):
        off = []
        for r in rs:
            v = tail(r["a"], r["excel"] / r["scale"], r["upper"])
            off.append(round((v - r["p"]) / math.ulp(r["p"])) if v != r["p"] else 0)
        print(f"{name}: our double cdf at Excel's quantile is p on {off.count(0)}, below on "
              f"{sum(o < 0 for o in off)}, above on {sum(o > 0 for o in off)}")
    for r in s112:
        r["unit_cross"] = crossing(r)
    cands = {
        "crossing (correctly rounded quantile)": lambda r: r["unit_cross"],
        "statrs' search": statrs_search,
        "Newton from a, stop at cdf = p": lambda r: newton(r, r["a"], 0),
        "Newton from a, stop at step < 1E-15": lambda r: newton(r, r["a"], 1),
        "Newton from Wilson-Hilferty, stop at cdf = p": lambda r: newton(r, wilson_hilferty(r), 0),
        "Newton from Wilson-Hilferty, stop at step < 1E-15": lambda r: newton(r, wilson_hilferty(r), 1),
        "Newton from crossing x (1 + 1E-3), stop at cdf = p": lambda r: newton(r, r["unit_cross"] * 1.001, 0),
        "Newton from crossing x (1 - 1E-3), stop at cdf = p": lambda r: newton(r, r["unit_cross"] * 0.999, 0),
        "Newton on ln cdf from a": lambda r: newton_log_cdf(r, r["a"]),
        "Newton in ln x from a": lambda r: newton_log_x(r, r["a"]),
        "bisection from [0, 2^k], stop at cdf = p": bisect,
    }
    print(f"{'candidate':55s} {'S: bit / 4ulp / vis':>22s} {'S99: bit / 4ulp / vis':>24s}")
    for name, f in cands.items():
        res = {}
        for r in s112:
            x = f(r) * r["scale"]
            e = r["excel"]
            res[r["cell"]] = (x == e, abs(x - e) <= 4 * math.ulp(e), vis(x, e))
        def score(rs):
            t = [res[r["cell"]] for r in rs]
            return "%3d / %3d / %3d" % (sum(b for b, _, _ in t), sum(u for _, u, _ in t), sum(v for _, _, v in t))
        print(f"{name:55s} {score(s112):>22s} {score(s99):>24s}")


if __name__ == "__main__":
    main()
