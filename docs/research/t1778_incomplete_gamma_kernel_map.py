"""Excel's erf/normal/chi-square/Poisson family runs on one incomplete-gamma kernel (T-1778).

Usage (from tools/, so the fit lab imports):

    uv run python ../docs/research/t1778_incomplete_gamma_kernel_map.py

Reads only oracle sidecars of the probes-numeric `kmyst-*` cases and prints the numbers quoted in
docs/excel-semantics/statistics.md ("One incomplete-gamma kernel ..."):
1. bit-exact identity counts between Excel functions (NORM.S.DIST vs ERFC.PRECISE, ERF vs
   GAMMA.DIST, the kernel-map families);
2. Excel's erf constant for tiny x: ERF.PRECISE(2^-k) * 2^k;
3. GAMMA.DIST at tiny z: Excel's relative error against z^a/Gamma(a+1) next to the rounding of
   fl(a * fl(ln z)), and the density's relative error next to the CDF's;
4. ERFC.PRECISE for 0.5 <= x <= 27: Excel's error before and after removing the naive exp(-fl(x*x))
   rounding;
5. the tiny-x erf split fl(fl(x*A) + fl(x*B)) and the GAMMA.DIST density built from the CDF.
"""

from __future__ import annotations

import decimal
import json
import math
import statistics
from collections import defaultdict
from pathlib import Path

# The fit lab's 80-digit exact erfc (private to norm_inv; this script tracks its names).
from rustsheet_tools.fitlab.kernels.norm_inv import _CTX, _erfc

CASES = Path(__file__).resolve().parents[2] / "eval/suites/probes-numeric/cases"
D = decimal.Decimal

Cells = dict[str, object]


def load(name: str) -> tuple[Cells, Cells]:
    """Case cells and oracle values of one probe case."""
    case = json.loads((CASES / f"{name}.case.json").read_text())["sheets"][0]["cells"]
    raw = json.loads((CASES / f"{name}.oracle.json").read_text())["sheets"]["Calc"]
    return case, {k: v.get("v") for k, v in raw.items()}


def column(case: Cells, values: Cells, col: str, rows: range) -> list[tuple[float, object]]:
    """(A{r}, value of col{r}) for the rows of a case that have both."""
    return [
        (float(case[f"A{r}"]), values[f"{col}{r}"]) for r in rows if f"A{r}" in case and f"{col}{r}" in values
    ]  # type: ignore[arg-type]


def same(values: Cells, rows: list[int], a: str, b: str) -> int:
    return sum(values.get(f"{a}{r}") == values.get(f"{b}{r}") for r in rows)


def identities() -> None:
    print("1. Bit-exact identities (matches / rows)")
    for name in ("kmyst-norm-erfc-a", "kmyst-norm-erfc-b"):
        case, val = load(name)
        rows = [r for r in range(1, 2000) if f"A{r}" in case]
        print(
            f"  {name}: NORM.S.DIST(x) = ERFC.PRECISE(-x*sqrt(0.5))/2: {same(val, rows, 'E', 'F')}/{len(rows)}"
        )
    # Reflections, from the same rows: D = x*sqrt(0.5), F = ERFC.PRECISE(-D)/2, K = ERF.PRECISE(D).
    below, above, odd = [0, 0], [0, 0], [0, 0]
    erf_at: dict[float, float] = {}
    for name in ("kmyst-norm-erfc-a", "kmyst-norm-erfc-b"):
        case, val = load(name)
        for r in range(1, 2000):
            if f"D{r}" not in case or not isinstance(val.get(f"K{r}"), float):
                continue
            d, f, k = float(case[f"D{r}"]), float(val[f"F{r}"]), float(val[f"K{r}"])  # type: ignore[arg-type]
            erf_at[d] = k
            if 0 <= d < 0.5:
                below[0] += 1
                below[1] += 1 + k == 2 * f
            elif d >= 0.5:
                above[0] += 1
                above[1] += k == 2 * f - 1
    for d, k in erf_at.items():
        if d > 0 and -d in erf_at:
            odd[0] += 1
            odd[1] += erf_at[-d] == -k
    print(
        f"  ERFC(-d) = fl(1+ERF(d)) for 0 <= d < 0.5: {below[1]}/{below[0]}; "
        f"ERF(d) = ERFC(-d)-1 for d >= 0.5: {above[1]}/{above[0]}"
    )
    print(f"  ERF(-d) = -ERF(d): {odd[1]}/{odd[0]} pairs")
    case, val = load("kmyst-erf-gamma")
    small = [r for r in range(1, 601) if float(case[f"A{r}"]) < 0.5]  # type: ignore[arg-type]
    large = [r for r in range(1, 601) if float(case[f"A{r}"]) >= 0.5]  # type: ignore[arg-type]
    for label, rows in (("x < 0.5", small), ("x >= 0.5", large)):
        print(
            f"  kmyst-erf-gamma {label}: ERF = GAMMA.DIST(x^2,0.5,1,TRUE) {same(val, rows, 'C', 'D')}/{len(rows)}, "
            f"ERF = CHISQ.DIST(2x^2,1,TRUE) {same(val, rows, 'C', 'E')}/{len(rows)}, "
            f"ERFC = CHISQ.DIST.RT(2x^2,1) {same(val, rows, 'F', 'H')}/{len(rows)}, "
            f"ERFC = 1-GAMMA.DIST {same(val, rows, 'F', 'G')}/{len(rows)}"
        )
    case, val = load("kmyst-kernel-map")
    families: dict[str, list[int]] = defaultdict(list)
    for r in range(1, 601):
        families[str(case[f"H{r}"])].append(r)
    for fam, rows in families.items():
        # gscale's G and I columns hold densities next to E's CDF: no identity to test there.
        second = "" if fam == "gscale" else f", E=G {same(val, rows, 'E', 'G')}/{len(rows)}"
        print(f"  kernel map {fam:8s}: E=F {same(val, rows, 'E', 'F')}/{len(rows)}{second}")


def erf_constant() -> None:
    case, val = load("kmyst-erf-tiny")
    ratios = {float(case[f"A{r}"]): float(val[f"B{r}"]) / float(case[f"A{r}"]) for r in range(1, 80)}  # type: ignore[arg-type]
    tiny = sorted({v for x, v in ratios.items() if x <= 2.0**-27})
    print("2. ERF.PRECISE(2^-k)*2^k for k >= 27:", [repr(v) for v in tiny])
    print(
        f"   fl(2/sqrt(pi)) = {2 / math.sqrt(math.pi)!r}; one ulp up = {math.nextafter(2 / math.sqrt(math.pi), 2)!r}"
    )


def gamma_prefix() -> None:
    case, val = load("kmyst-gamma-tiny")
    print(
        "3. GAMMA.DIST(z,a,1,TRUE), z = m*2^-60: Excel's relative error (1e-16 units) vs the fl(a*fl(ln z)) rounding"
    )
    cdf_col = {2: "E", 3: "F", 10: "J"}
    pdf_col = {2: "M", 3: "N"}
    for a, col in cdf_col.items():
        rows = []
        for r in range(1, 9):
            z = float(case[f"A{r}"])  # type: ignore[arg-type]
            excel = float(val[f"{col}{r}"])  # type: ignore[arg-type]
            exact = _CTX.exp(D(a) * _CTX.ln(D(z))) / D(math.factorial(a))
            ln_z = math.log(z)
            model = D(a) * (D(ln_z) - _CTX.ln(D(z))) + (D(a * ln_z) - D(a) * D(ln_z))
            rel = (D(excel) - exact) / exact
            line = f"{float(rel) * 1e16:+8.2f} ({float(model) * 1e16:+8.2f})"
            if a in pdf_col:
                pdf = float(val[f"{pdf_col[a]}{r}"])  # type: ignore[arg-type]
                pdf_exact = exact * D(a) / D(z)
                line += f" pdf {float((D(pdf) - pdf_exact) / pdf_exact) * 1e16:+8.2f}"
            rows.append(line)
        print(f"   a={a}: " + " | ".join(rows))


def erfc_tail() -> None:
    case, val = load("kmyst-erfc-dense")
    bins: dict[int, list[tuple[float, float]]] = defaultdict(list)
    for w, e in column(case, val, "B", range(1, 2001)):
        if not isinstance(e, float) or e < 2.3e-308:
            continue
        exact = _erfc(D(w))
        ulp = D(math.ulp(e))
        naive = D(math.exp(-(w * w))) * exact / _CTX.exp(-D(w) * D(w))
        bins[4 if w >= 4 else 0].append((float((D(e) - exact) / ulp), float((D(e) - naive) / ulp)))
    print("4. ERFC.PRECISE: error in ulps against erfc(x), then against exp(-fl(x*x)) * erfc(x)*exp(x^2)")
    for b, pairs in sorted(bins.items()):
        raw = [p[0] for p in pairs]
        res = [p[1] for p in pairs]
        print(
            f"   x {'>= 4' if b else '0.5..4'}: n={len(pairs)} raw sd {statistics.pstdev(raw):.2f}, "
            f"after naive exp sd {statistics.pstdev(res):.2f} max {max(map(abs, res)):.2f}"
        )


def erf_tiny_split() -> None:
    """Excel's erf below about 1E-8: fl(fl(x*A) + fl(x*B)), and the density = fl(CDF * fl(a/z))."""
    split_a, split_b = 1.1170334248975802, 0.011345742197932508
    hits = total = 0
    for name, rows in (("kmyst-erf-mantissa", range(1, 401)), ("kmyst-erf-consecutive", range(1, 2049))):
        case, val = load(name)
        for x, e in column(case, val, "B", rows):
            total += 1
            hits += x * split_a + x * split_b == e
    print(f"5. ERF.PRECISE = fl(fl(x*A) + fl(x*B)) below 1E-8: {hits}/{total}")
    case, val = load("kmyst-gamma-tiny")
    for a, cdf, pdf in ((2, "E", "M"), (3, "F", "N"), (1.5, "G", "L")):
        rows = range(1, 101)
        n = sum(
            float(val[f"{cdf}{r}"]) * (a / float(case[f"A{r}"])) == float(val[f"{pdf}{r}"])  # type: ignore[arg-type]
            for r in rows
        )
        print(f"   GAMMA.DIST(z,{a},1,FALSE) = fl(CDF * fl(a/z)) at z = m*2^-60: {n}/100")


if __name__ == "__main__":
    identities()
    erf_constant()
    gamma_prefix()
    erfc_tail()
    erf_tiny_split()
