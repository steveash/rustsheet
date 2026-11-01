"""T-3447 fit attempt 1: which roundings make up Excel's BINOM.DIST pmf error at a large n.

Data: the large-n BINOM.DIST runs of consecutive doubles p in probes-numeric/t2513-r3-binom-pmf-n (column B,
n 1E4 to 1E7, cells where Excel is not 0), Excel's values from the oracle file, the engine's from
`rustsheet-eval show --all probes-numeric/t2513-r3-binom-pmf-n.case.json` saved to SHOW (argument 1).

Reference: Loader's dbinom_raw in 60-digit math at the exact n p and n (1 - p). Each cell's relative error
(Excel's, then the engine's) is divided by |d| 2^-53, d = k - fl(n p), the size of the roundings in question,
and regressed (least squares) on the first-order effect of candidate roundings on each bd0 side:
  M    fl(n p) and fl(n fl(1 - p)) against the exact products (dbd0/dM = -d/M)
  v    v = fl(d / fl(k + M)) against d / (k + M), entering as 2k dv (an `fl(2kv) - d` form)
  ej   fl(2 k v) - 2 k v with v stored, the engine's `ph` term (loader.rs excel_bd0_series)
  s    fl(k + M) alone; sv, Mv, dv: the roundings of s v, M v and d v
On the engine the fit returns its own model (M 1, 1; ej -1, -1; residual rms 0.04), which validates the method.

Usage: tools/.venv/bin/python -I docs/research/t3447/pmf_regress.py SHOW_FILE   (run from the repo root)
"""

from __future__ import annotations

import json
import math
import re
import sys

import mpmath
from mpmath import exp, log, loggamma, mpf, pi

mpmath.mp.dps = 60
CASE = "eval/suites/probes-numeric/cases/t2513-r3-binom-pmf-n"
NAMES = ["M", "v", "ej", "s", "sv", "Mv", "dv"]


def rnd(x: mpf, bits: int) -> mpf:
    """Round x to a binary float of `bits` significand bits, to nearest even."""
    return mpf(mpmath.libmp.mpf_pos(mpf(x)._mpf_, bits, "n"))


def stirlerr(n: float) -> mpf:
    m = mpf(n)
    return loggamma(m + 1) - (m + mpf(1) / 2) * log(m) + m - log(2 * pi) / 2


def bd0(k: float, m: mpf) -> mpf:
    return mpf(k) * log(mpf(k) / m) + m - k


def literal(text: str) -> float:
    """A probe's p literal, `(a*2^-e+b*2^-f)` or a decimal, as the double Excel builds (one rounding)."""
    total = mpf(0)
    for term in text.strip("()").split("+"):
        if "*2^" in term:
            mant, power = term.split("*2^")
            total += mpf(int(mant)) * mpf(2) ** int(power)
        else:
            total += mpf(term)
    return float(total)


def side(k: float, exact_m: mpf, m: mpf) -> list[mpf]:
    """First-order bd0 changes of each candidate rounding on one side (k against M)."""
    kk = mpf(k)
    d = kk - m
    s = rnd(kk + m, 53)
    v_exact = d / (kk + m)
    v = rnd(d / s, 53)
    ej = 2 * kk * v
    return [
        -(d / exact_m) * (m - exact_m),
        2 * kk * (v - v_exact),
        rnd(ej, 53) - ej,
        2 * kk * (d / s - v_exact),
        rnd(s * v, 53) - s * v,
        rnd(m * v, 53) - m * v,
        rnd(d * v, 53) - d * v,
    ]


def lstsq(xs: list[list[float]], ys: list[float], cols: list[int]) -> tuple[list[float], float]:
    a = mpmath.matrix(len(cols), len(cols))
    b = mpmath.matrix(len(cols), 1)
    for x, y in zip(xs, ys):
        for i, ci in enumerate(cols):
            b[i] += x[ci] * y
            for j, cj in enumerate(cols):
                a[i, j] += x[ci] * x[cj]
    coef = [float(c) for c in mpmath.lu_solve(a, b)]
    resid = [y - sum(c * x[ci] for c, ci in zip(coef, cols)) for x, y in zip(xs, ys)]
    return coef, math.sqrt(sum(r * r for r in resid) / len(resid))


def main() -> None:
    cells = json.load(open(CASE + ".case.json"))["sheets"][0]["cells"]
    oracle = json.load(open(CASE + ".oracle.json"))["sheets"]["Calc"]
    shown = open(sys.argv[1]).read()
    engine = {m[1]: float(m[2]) for m in re.finditer(r"!(B\d+).*?\n.*?\n.*?\n\s+actual\s*:\s*(\S+)", shown)}
    xs: list[list[float]] = []
    ys: dict[str, list[float]] = {"excel": [], "engine": []}
    for addr, formula in cells.items():
        m = re.match(r"=BINOM\.DIST\((\d+),(\d+),(.*),FALSE\)$", formula)
        if not (addr.startswith("B") and m and int(m[2]) >= 10000 and int(m[1]) > 100):
            continue
        excel = oracle.get(addr, {})
        if excel.get("t") != "n" or excel["v"] == 0:
            continue
        k, n, p = float(m[1]), float(m[2]), literal(m[3])
        q = float(1 - mpf(p))
        nk = n - k
        exact_p, exact_q = mpf(n) * mpf(p), mpf(n) * (1 - mpf(p))
        m_p, m_q = rnd(mpf(n) * mpf(p), 53), rnd(mpf(n) * mpf(q), 53)
        d = abs(float(k - m_p))
        if d < 20:
            continue
        lf = log(2 * pi) + log(mpf(k)) + log(mpf(nk)) - log(mpf(n))
        ref = exp(stirlerr(n) - stirlerr(k) - stirlerr(nk) - bd0(k, exact_p) - bd0(nk, exact_q) - lf / 2)
        scale = 1 / (d * 2.0**-53)
        xs.append([-float(t) * scale for t in side(k, exact_p, m_p) + side(nk, exact_q, m_q)])
        ys["excel"].append(float((excel["v"] - ref) / ref) * scale)
        ys["engine"].append(float((engine[addr] - ref) / ref) * scale)
    print(f"{len(xs)} cells")
    width = len(NAMES)
    for label, y in ys.items():
        print(f"{label}: rms {math.sqrt(sum(t * t for t in y) / len(y)):.2f} before any term")
        for extra in [[], [1], [3], [4], [5], [6]]:
            cols = [0, width, 2, width + 2] + [c for e in extra for c in (e, width + e)]
            coef, rms = lstsq(xs, y, cols)
            names = [NAMES[c % width] + "pq"[c // width] for c in cols]
            print("  " + " ".join(f"{nm} {c:+.2f}" for nm, c in zip(names, coef)) + f"  rms {rms:.2f}")


if __name__ == "__main__":
    main()
