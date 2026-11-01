"""T-2924: structural tests on Excel's ln B(a, b) for 2.5 <= a <= 8 < b (probes/t2829-ftest-ln-beta-grid).

Usage: python structure.py (needs mpmath, numpy and scipy on x86-64, as ../t2829/ln_beta_map.py does).

Each test rules out a whole family of forms at once instead of scoring one candidate at a time. L_excel is the
stored double the T-2829 map reads off each of the 99 cells: fl(accurate ln B) stepped by its integer k.

1. Lattice: L = fl(g(a) o (g(b) o g(a+b))) with g(v) any double within 2 ulp of lnGamma(v), seven sum orders
   (double, x87-stored, one extended sum). g(b) and g(a+b) sit on a grid 4 ulp(L) wide, so L's last bits come
   from g(a) alone; prints how many cells no table can reach.
2. Reduction: for integer a, TOMS betaln reduces a to a0 = 1, so L = fl(LN(w) + A(b)) with w the product of the
   reduction factors and A(b) = gamln(1) + algdiv(1, b) depending on b only. For each w variant (and the exact
   ln of the product) prints for how many of the 11 b values one A(b) fits all six integer a.
3. Separable: L = fl(lnB + f(a) + h(b) + q(a+b)) with f, h, q arbitrary (any per-argument lnGamma table, in any
   precision, summed once). Prints whether a linear program finds such f, h, q; controls: two separable forms
   (feasible) and random k (infeasible).
4. The most cells a separable error can explain (mixed-integer program), against random k.
"""

import itertools
import math
import random
import sys
from pathlib import Path

import numpy as np
from scipy.optimize import Bounds, LinearConstraint, linprog, milp

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "t2829"))
import ln_beta_map as M  # noqa: E402 - needs the sys.path entry set above
import mpmath  # noqa: E402
from mpmath import mpf  # noqa: E402

B, LN, sx, LD = M.B, M.B.LN, M.B.sx, np.longdouble
SCALE = mpf(10) ** 15


def nudge(x: float, k: int) -> float:
    for _ in range(abs(k)):
        x = math.nextafter(x, math.inf if k > 0 else -math.inf)
    return x


ROWS = [(a, b, round(k)) for a, b, k in M.grid()]
ACC = {(a, b): float(M.accurate(a, b)) for a, b, _ in ROWS}
EXCEL = {(a, b): nudge(ACC[a, b], k) for a, b, k in ROWS}
EXACT = {(a, b): mpmath.log(mpmath.beta(mpf(a), mpf(b))) for a, b, _ in ROWS}
INT = [r for r in ROWS if float(r[0]).is_integer()]


def lgcr(v: float) -> float:
    return float(mpmath.loggamma(mpf(v)))


def lattice(r: int = 2) -> None:
    near = {}
    for v in {v for a, b, _ in ROWS for v in (a, b, a + b)}:
        near[v] = [nudge(lgcr(v), k) for k in range(-r, r + 1)]
    orders = {
        "a+(b-ab)": lambda p, q, s: p + (q - s),
        "(a+b)-ab": lambda p, q, s: (p + q) - s,
        "(a-ab)+b": lambda p, q, s: (p - s) + q,
        "b+(a-ab)": lambda p, q, s: q + (p - s),
        "x87 a+(b-ab)": lambda p, q, s: sx("+", p, sx("-", q, s)),
        "x87 (a+b)-ab": lambda p, q, s: sx("-", sx("+", p, q), s),
        "one extended sum": lambda p, q, s: float(LD(p) + LD(q) - LD(s)),
    }
    for name, f in orders.items():
        dead = sum(
            not any(f(p, q, s) == EXCEL[a, b] for p, q, s in itertools.product(near[a], near[b], near[a + b]))
            for a, b, _ in ROWS
        )
        print(f"  {name:18} {dead:2} of 99 cells unreachable with every g within {r} ulp")


def reduction() -> None:
    factors = {
        "h/(1+h)": lambda t, b: (t / b) / (t / b + 1.0),
        "t/(t+b)": lambda t, b: t / (t + b),
        "t/(t+b) stored": lambda t, b: sx("/", t, sx("+", t, b)),
    }

    def ln_w(a: float, b: float, f, mode: str, asc: bool):
        seq = [float(t) for t in range(int(a) - 1, 0, -1)]
        w = LD(1) if mode == "ext" else 1.0
        for t in seq[::-1] if asc else seq:
            w = (
                w * f(LD(t), LD(b))
                if mode == "ext"
                else (w * f(t, b) if mode == "double" else sx("*", w, f(t, b)))
            )
        return mpf(LN(float(w)))

    variants = [
        (
            f"{fn} {mode} {'up' if asc else 'down'}",
            lambda a, b, f=f, mode=mode, asc=asc: ln_w(a, b, f, mode, asc),
        )
        for (fn, f), mode, asc in itertools.product(factors.items(), ("double", "x87", "ext"), (False, True))
    ]
    variants.append(
        (
            "exact ln of the product",
            lambda a, b: mpmath.log(mpmath.fprod(mpf(t) / (t + b) for t in range(1, int(a)))),
        )
    )
    for name, lw in variants:
        fits = 0
        for b in sorted({b for _, b, _ in INT}):
            lo, hi = -mpmath.inf, mpmath.inf
            for a, _, _ in (r for r in INT if r[1] == b):
                half = mpf(math.ulp(EXCEL[a, b])) / 2
                lo, hi = max(lo, EXCEL[a, b] - half - lw(a, b)), min(hi, EXCEL[a, b] + half - lw(a, b))
            fits += lo <= hi
        print(f"  {fits:2} of 11 b values fit one A(b)   w: {name}")


def separable_rows(target: dict, rows: list):
    """Rows of interval constraints lo <= f(a) + h(b) + q(a+b) <= hi, in units of 1E-15."""
    cols = {}
    for a, b, _ in rows:
        for key in (("f", a), ("h", b), ("q", a + b)):
            cols.setdefault(key, len(cols))
    mat, lo, hi = [], [], []
    for a, b, _ in rows:
        half = mpf(math.ulp(target[a, b])) / 2
        r = np.zeros(len(cols))
        for key in (("f", a), ("h", b), ("q", a + b)):
            r[cols[key]] = 1.0
        mat.append(r)
        lo.append(float((target[a, b] - half - EXACT[a, b]) * SCALE))
        hi.append(float((target[a, b] + half - EXACT[a, b]) * SCALE))
    return np.array(mat), np.array(lo), np.array(hi)


def separable(target: dict, rows: list) -> bool:
    mat, lo, hi = separable_rows(target, rows)
    res = linprog(
        np.zeros(mat.shape[1]),
        A_ub=np.vstack([mat, -mat]),
        b_ub=np.concatenate([hi, -lo]),
        bounds=[(None, None)] * mat.shape[1],
        method="highs",
    )
    return res.status == 0


def most_cells(target: dict, bound: float = 20.0) -> int:
    """Most cells a separable error (each term within `bound` E-15) explains; the rest get a big-M release."""
    mat, lo, hi = separable_rows(target, ROWS)
    n, nv, big = len(ROWS), mat.shape[1], 200.0
    rel = np.eye(n) * big
    cons = [
        LinearConstraint(np.hstack([mat, rel]), lo, np.inf),
        LinearConstraint(np.hstack([mat, -rel]), -np.inf, hi),
    ]
    res = milp(
        np.concatenate([np.zeros(nv), np.ones(n)]),
        constraints=cons,
        integrality=np.concatenate([np.zeros(nv), np.ones(n)]),
        bounds=Bounds(
            np.concatenate([-bound * np.ones(nv), np.zeros(n)]),
            np.concatenate([bound * np.ones(nv), np.ones(n)]),
        ),
        options={"time_limit": 120},
    )
    return n - round(res.fun)


def main() -> None:
    print("1. lattice")
    lattice()
    print("2. reduction (integer a)")
    reduction()
    print("3. separable")
    random.seed(1)
    controls = {
        "Excel": EXCEL,
        "control: one extended sum of correctly rounded lnGamma": {
            (a, b): float(LD(lgcr(a)) + LD(lgcr(b)) - LD(lgcr(a + b))) for a, b, _ in ROWS
        },
        "control: TOMS betaln (Excel LN, GAMMALN)": {
            (a, b): B.Kernel(B.BEST).betaln(a, b) for a, b, _ in ROWS
        },
        "control: random k in -2..2": {(a, b): nudge(ACC[a, b], random.randint(-2, 2)) for a, b, _ in ROWS},
    }
    for name, target in controls.items():
        print(f"  {name}: all 99 {separable(target, ROWS)}, integer a {separable(target, INT)}")
    print("4. most cells a separable error explains")
    random.seed(2)
    print(f"  Excel {most_cells(EXCEL)} of 99")
    print(
        f"  random k in -2..2: {most_cells({(a, b): nudge(ACC[a, b], random.randint(-2, 2)) for a, b, _ in ROWS})} of 99"
    )


if __name__ == "__main__":
    main()
