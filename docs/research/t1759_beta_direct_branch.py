"""BETA.DIST's direct (unreflected) branch: the T-1759 analyses behind statistics.md.

Usage (from tools/, so the fit lab imports):

    uv run python ../docs/research/t1759_beta_direct_branch.py

Four reports over the beta-dist sweep (fit `beta-dist`, fitlab/kernels/beta.py), on the
de-duplicated (x, a, b) points with numeric arguments:
1. bit-exact matches per branch (closed form, direct, reflected; LibreOffice's pdf/explog split)
   for the best Cephes combination, LibreOffice + Excel's closed forms and the correctly rounded
   value;
2. Excel's relative error against the exact I_x(a, b) on the direct branch, compared with two
   models of where it comes from: a*err(ln x), and the total rounding of fl(a * fl(ln x));
3. 360 combinations of continued fraction (Cephes, LibreOffice, Numerical Recipes), prefactor form
   and operation order on the direct branch;
4. the same with Excel's own GAMMALN/GAMMA values (T-1042 sweeps) as the Beta-function source.
"""

from __future__ import annotations

import decimal
import itertools
import math
from collections import Counter, defaultdict
from collections.abc import Callable

from rustsheet_tools.fitlab.kernels import beta as kb
from rustsheet_tools.fitlab.kernels import registry
from rustsheet_tools.fitlab.sweep import load_sweep

Point = tuple[float, float, float]
CTX = decimal.Context(prec=70)


def sweep_points() -> dict[Point, float]:
    fit = registry()["beta-dist"]
    sweep = load_sweep(fit.cases, fit.pattern, None, False)
    out: dict[Point, float] = {}
    for cell in sweep.cells:
        if all(isinstance(v, float) for v in cell.args):
            x, a, b = (float(v) for v in cell.args[:3])  # type: ignore[arg-type]
            out[(x, a, b)] = cell.expected
    return out


def branch(x: float, a: float, b: float) -> str:
    if a == 1.0 or b == 1.0:
        return "closed form"
    reflect = x > a / (a + b)
    if reflect:
        a, b = b, a
    p, q = a / (a + b), b / (a + b)
    shape = "pdf" if a > 1 and b > 1 and p < 0.97 and q < 0.97 else "explog"
    return ("reflected " if reflect else "direct ") + shape


def exact_ibeta(x: float, a: float, b: float) -> decimal.Decimal:
    """I_x(a, b) unrounded, with x < (a+1)/(a+b+2) (the direct branch): NR's fraction at 70 digits."""
    dx, da, db = decimal.Decimal(x), decimal.Decimal(a), decimal.Decimal(b)
    lbt = kb.ln_gamma_decimal(da + db) - kb.ln_gamma_decimal(da) - kb.ln_gamma_decimal(db)
    lbt += da * CTX.ln(dx) + db * CTX.ln(1 - dx)
    qab, qap, qam = da + db, da + 1, da - 1
    cc, d = decimal.Decimal(1), 1 / (1 - qab * dx / qap)
    h = d
    for m in range(1, 5000):
        m2 = 2 * m
        for aa in (
            m * (db - m) * dx / ((qam + m2) * (da + m2)),
            -(da + m) * (qab + m) * dx / ((da + m2) * (qap + m2)),
        ):
            d = 1 / (1 + aa * d)
            cc = 1 + aa / cc
            delta = d * cc
            h *= delta
        if abs(delta - 1) < decimal.Decimal(10) ** -62:
            break
    return CTX.exp(lbt) * h / da


def report_branches(points: dict[Point, float]) -> None:
    print("1. bit-exact points per branch")
    table: dict[str, Counter[str]] = defaultdict(Counter)
    for (x, a, b), expected in points.items():
        if not 0.0 < x < 1.0:
            continue
        row = table[branch(x, a, b)]
        row["points"] += 1
        row["best combination"] += kb.best_combination(x, a, b) == expected
        row["LibreOffice + closed forms"] += (
            kb.libreoffice_beta_dist(x, a, b, excel_closed_forms=True) == expected
        )
        row["correctly rounded"] += kb.ibeta_reference(x, a, b) == expected
    for name, row in sorted(table.items()):
        print(f"   {name:18s} {dict(row)}")


def direct_by_pair(points: dict[Point, float]) -> dict[tuple[float, float], dict[float, float]]:
    out: dict[tuple[float, float], dict[float, float]] = defaultdict(dict)
    for (x, a, b), expected in points.items():
        if branch(x, a, b).startswith("direct") and 0.0 < x < 1.0:
            out[(a, b)][x] = expected
    return out


def report_error_models(points: dict[Point, float]) -> None:
    print("2. direct-branch relative error (x 1e16) and the residual after each model")

    def a_err_ln(x: float, a: float) -> float:
        return float((decimal.Decimal(math.log(x)) - CTX.ln(decimal.Decimal(x))) * decimal.Decimal(a))

    def product_rounding(x: float, a: float) -> float:
        return float(decimal.Decimal(a * math.log(x)) - decimal.Decimal(a) * CTX.ln(decimal.Decimal(x)))

    models: dict[str, Callable[[float, float], float]] = {
        "a*err(ln x)": a_err_ln,
        "fl(a*fl(ln x)) rounding": product_rounding,
    }
    flat: Counter[str] = Counter()
    pairs = 0
    for (a, b), xs in sorted(direct_by_pair(points).items()):
        if len(xs) < 2:
            continue
        pairs += 1
        errors = {
            x: float((decimal.Decimal(e) - exact_ibeta(x, a, b)) / exact_ibeta(x, a, b))
            for x, e in xs.items()
        }
        line = "  ".join(f"x={x:g}:{v * 1e16:8.2f}" for x, v in sorted(errors.items()))
        print(f"   a={a:<7g} b={b:<7g} {line}")
        for name, model in models.items():
            residual = [(v - model(x, a)) * 1e16 for x, v in errors.items()]
            flat[name] += max(residual) - min(residual) < 3.0
    for name in models:
        print(f"   {name:24s}: residual flat (spread < 3e-16) for {flat[name]} of {pairs} (a, b) pairs")


def _cf(name: str, x: float, a: float, b: float) -> float:
    if name == "Cephes":
        return kb.cephes_cf(a, b, x)
    if name == "LibreOffice":
        return kb._lo_cont_frac(x, a, b)
    return kb.nr_betacf(a, b, x)


def _ln_y(x: float, log1p: bool) -> float:
    return math.log1p(-x) if log1p else math.log(1.0 - x)


def report_combinations(points: dict[Point, float]) -> None:
    print("3. continued fraction x prefactor form x operation order (direct branch)")
    direct = {k: v for k, v in points.items() if branch(*k).startswith("direct") and 0.0 < k[0] < 1.0}
    ln_beta: dict[str, Callable[[float, float], float]] = {
        "lgamma": lambda a, b: math.lgamma(a) + math.lgamma(b) - math.lgamma(a + b),
        "LibreOffice": kb._lo_log_beta,
        "correctly rounded": lambda a, b: kb.ln_gamma_cr(a) + kb.ln_gamma_cr(b) - kb.ln_gamma_cr(a + b),
    }
    beta_fn: dict[str, Callable[[float, float], float]] = {
        "tgamma": lambda a, b: math.gamma(a) * math.gamma(b) / math.gamma(a + b),
        "LibreOffice": kb._lo_beta,
        "correctly rounded": lambda a, b: kb.gamma_cr(a) * kb.gamma_cr(b) / kb.gamma_cr(a + b),
    }
    forms: dict[str, Callable[[float, float, float], float]] = {}
    for (name, lb), log1p in itertools.product(ln_beta.items(), (False, True)):
        forms[f"exp(a lx + b ly - lnB[{name}]) log1p={log1p}"] = lambda x, a, b, lb=lb, l1=log1p: math.exp(
            a * math.log(x) + b * _ln_y(x, l1) - lb(a, b)
        )
        forms[f"exp(a lx + b ly)/exp(lnB[{name}]) log1p={log1p}"] = lambda x, a, b, lb=lb, l1=log1p: (
            math.exp(a * math.log(x) + b * _ln_y(x, l1)) / math.exp(lb(a, b))
        )
    for (name, bf), log1p in itertools.product(beta_fn.items(), (False, True)):
        forms[f"exp(a lx)*exp(b ly)/B[{name}] log1p={log1p}"] = lambda x, a, b, bf=bf, l1=log1p: (
            math.exp(a * math.log(x)) * math.exp(b * _ln_y(x, l1)) / bf(a, b)
        )
        forms[f"exp(a lx + b ly)/B[{name}] log1p={log1p}"] = lambda x, a, b, bf=bf, l1=log1p: (
            math.exp(a * math.log(x) + b * _ln_y(x, l1)) / bf(a, b)
        )
    for name, bf in beta_fn.items():
        forms[f"x^a*y^b/B[{name}]"] = lambda x, a, b, bf=bf: float(x**a * (1 - x) ** b) / bf(a, b)
        forms[f"LibreOffice pdf*x*y [B {name}]"] = lambda x, a, b, bf=bf: (
            float(x ** (a - 1) * (0.5 - x + 0.5) ** (b - 1)) / bf(a, b) * x * ((0.5 - x) + 0.5)
        )
    orders: dict[str, Callable[[float, float, float], float]] = {
        "P*w/a": lambda p, w, a: p * w / a,
        "P/a*w": lambda p, w, a: p / a * w,
        "w/a*P": lambda p, w, a: w / a * p,
        "P*(w/a)": lambda p, w, a: p * (w / a),
    }
    results = []
    for cf, (fname, form), (oname, order) in itertools.product(
        ("Cephes", "LibreOffice", "NR"), forms.items(), orders.items()
    ):
        hits = 0
        for (x, a, b), expected in direct.items():
            try:
                hits += order(form(x, a, b), _cf(cf, x, a, b), a) == expected
            except (ValueError, OverflowError, ZeroDivisionError):
                continue
        results.append((hits, cf, fname, oname))
    print(f"   {len(results)} combinations over {len(direct)} points; best:")
    for row in sorted(results, reverse=True)[:5]:
        print(f"   {row}")


def report_excel_gamma(points: dict[Point, float]) -> None:
    print(
        "4. Excel's own GAMMA (T-1042) as the Beta-function source (all branches, integer/half-integer shapes)"
    )
    gamma: dict[float, float] = {}
    sweep = load_sweep(
        [
            "probes-numeric/t1042-gamma-int-1-90",
            "probes-numeric/t1042-gamma-int-91-171",
            "probes-numeric/t1042-gamma-half-int",
        ],
        r"^=GAMMA\((.*)\)$",
        None,
        False,
    )
    for cell in sweep.cells:
        if isinstance(cell.args[0], float):
            gamma[cell.args[0]] = cell.expected
    usable = {
        k: v
        for k, v in points.items()
        if k[1] != 1 and k[2] != 1 and 0 < k[0] < 1 and all(s in gamma for s in (k[1], k[2], k[1] + k[2]))
    }
    best = 0
    for cf, log1p in itertools.product(("Cephes", "LibreOffice"), (False, True)):
        hits = 0
        for (x0, a0, b0), expected in usable.items():
            x, a, b, reflect = (1 - x0, b0, a0, True) if x0 > a0 / (a0 + b0) else (x0, a0, b0, False)
            prefactor = (
                math.exp(a * math.log(x))
                * math.exp(b * _ln_y(x, log1p))
                * (gamma[a + b] / (gamma[a] * gamma[b]))
            )
            value = prefactor / a * _cf(cf, x, a, b)
            hits += (1 - value if reflect else value) == expected
        best = max(best, hits)
    print(f"   best {best} of {len(usable)} points")


def main() -> None:
    points = sweep_points()
    report_branches(points)
    report_error_models(points)
    report_combinations(points)
    report_excel_gamma(points)


if __name__ == "__main__":
    main()
