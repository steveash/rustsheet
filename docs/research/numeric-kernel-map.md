# Numeric kernel map (T-1522, 2026-10-14)

Which shared numeric kernels the near-miss cells depend on, ranked by how many cells each could
fix. Decision 0056 makes fit work kernel-first in this order.

**Reproduce:** `make eval` on main at 49998df2, then
`python3 docs/research/numeric_kernel_map.py` (it reads `eval/runs/latest/report.json`).
The script attributes each failing cell to the kernel of every numeric function in its formula,
ignoring wrappers such as DATE, IF and INDEX. It counts only `from_excel` cells with a
`numeric_ulp` (≤ 4 ulp), `numeric_close` (beyond 4 ulp, within 1e-9 relative) or
`numeric_mismatch` verdict. **only this kernel** counts cells whose numeric functions all use
that kernel, so fixing the kernel alone can fix them. Re-run the script when the data changes;
the counts below are a snapshot.

Totals: 9,507 numeric near-miss cells (3,784 ulp, 4,604 close, 1,119 mismatch). The per-function
figures in T-1522's Context (about 9,400 pairs) count a cell once per function, so they differ
slightly.

| kernel | cells | only this kernel | ulp | close | mismatch | suites |
|---|---|---|---|---|---|---|
| annuity (1+r)^n | 1788 | 1517 | 785 | 952 | 51 | probes-numeric 1449, ironcalc-calc 315, probes 12 |
| normal-cdf/erfc | 1738 | 1360 | 1388 | 350 | 0 | probes-numeric 1655, ironcalc-calc 71, probes 12 |
| incomplete-beta | 1376 | 1375 | 222 | 1084 | 70 | probes-numeric 1143, ironcalc-calc 211, probes 21 |
| solvers | 929 | 658 | 58 | 746 | 125 | probes-numeric 552, probes 213, ironcalc-calc 162 |
| inverse-beta | 664 | 664 | 151 | 444 | 69 | probes-numeric 538, ironcalc-calc 117, probes 9 |
| normal-inverse | 584 | 206 | 567 | 16 | 1 | probes-numeric 476, ironcalc-calc 106, probes 2 |
| trig reduction | 483 | 481 | 41 | 166 | 276 | probes-numeric 419, ironcalc-calc 47, poi 13 |
| incomplete-gamma | 430 | 430 | 171 | 258 | 1 | probes-numeric 346, ironcalc-calc 75, probes 7 |
| gamma/lngamma | 410 | 408 | 150 | 259 | 1 | probes-numeric 361, ironcalc-calc 39, probes 8 |
| (no numeric kernel) | 385 | 385 | 81 | 40 | 264 | probes 284, ironcalc-calc 74, poi 27 |
| bessel | 374 | 374 | 0 | 176 | 198 | probes-numeric 348, ironcalc-calc 26 |
| inverse-gamma | 293 | 292 | 61 | 190 | 42 | probes-numeric 250, ironcalc-calc 39, probes 3 |
| hyperbolic | 176 | 175 | 123 | 39 | 14 | probes-numeric 107, ironcalc-calc 60, poi 5 |
| pow/exp/ln/sqrt | 126 | 126 | 93 | 33 | 0 | probes 64, probes-numeric 56, ironcalc-calc 5 |
| weibull/expon | 108 | 108 | 74 | 32 | 2 | probes-numeric 106, ironcalc-calc 2 |
| bond-pricing | 107 | 107 | 88 | 18 | 1 | ironcalc-calc 67, probes 39, gen-m9 1 |
| normal-pdf | 94 | 94 | 94 | 0 | 0 | probes-numeric 93, ironcalc-calc 1 |
| hypergeometric | 45 | 44 | 22 | 23 | 0 | probes-numeric 16, ironcalc-calc 13, probes 13 |
| complex | 38 | 38 | 9 | 23 | 6 | ironcalc-calc 38 |
| inverse trig | 12 | 12 | 11 | 1 | 0 | ironcalc-calc 6, poi 3, gen-m7 2 |

## Kernels, call sites and owner tasks

Rank by **only this kernel** (cells a kernel fix alone can move), then by dependencies: an
inverse is fitted after its CDF, and gamma before the incomplete beta/gamma prefactors.

| # | kernel | engine call site | owner tasks | notes |
|---|---|---|---|---|
| 1 | annuity (1+r)^n | `functions/financial/mod.rs` `compute_payment`, `compute_future_value`, `compute_ipmt`, `compute_ppmt`; NPV/MIRR loops | T-0901, T-1405 | Engine's own closed forms (no library). 952 `close` cells point to a different evaluation order or algebraic form, not rounding. Cheapest high-yield fit; also feeds the RATE solver. |
| 2 | normal CDF / erfc | `functions/statistical/special.rs` `erfc`, `erfc_cf`, `erf_series`, `normal_cdf` | T-0968, T-1186 | 80% of its misses are ≤ 4 ulp. The fit lab's ERFC example (T-1523) is the starting point; the x87 model (T-1524) is the main hypothesis. |
| 3 | incomplete beta | `statrs` `beta_reg` via `Beta`, `StudentsT`, `FisherSnedecor`, `Binomial`, `NegativeBinomial` | T-0970, T-0969, T-0927 | 1,084 `close`: statrs' continued fraction differs from Excel's algorithm, not just its rounding. Needs our own kernel, which the inverse-beta rank depends on. |
| 4 | solvers | `functions/financial/financial_util.rs` `compute_rate`, `compute_irr`, `compute_xirr`; YIELD in `financial_bonds.rs` | T-0916 (T-1525 probes first) | 746 `close`: a different iterate or stop. T-1525 fits the trace. RATE also depends on rank 1. |
| 5 | inverse beta | statrs `inverse_cdf` (bisection or Newton on rank 3) | T-0974, T-0967 | After rank 3. |
| 6 | trig reduction | `functions/math_and_trigonometry` SIN/COS/TAN family, PI | T-0915, T-1201 | 276 `mismatch` near multiples of pi: argument reduction differs. |
| 7 | incomplete gamma | statrs `gamma_lr` / `gamma_ur` via `Gamma`, `ChiSquared`, `Poisson` | T-0971 | After rank 9. |
| 8 | gamma / ln gamma | `special.rs` `ln_gamma`; `statrs::function::gamma::gamma` | T-0973 | Prefactor of ranks 3 and 7, so fitting it first may move those too. |
| 9 | Bessel | `functions/engineering/bessel.rs` | T-0908 | 198 `mismatch` at large arguments. |
| 10 | inverse gamma | statrs `inverse_cdf` on rank 7 | T-0972 | After rank 7. |
| 11 | normal inverse | statrs `Normal::inverse_cdf` | T-0925 | 97% ≤ 4 ulp; Wichura AS241 is the named candidate. |
| 12 | hyperbolic | `math_and_trigonometry/hyperbolic.rs` | T-0917 | Mostly ≤ 4 ulp. |
| 13 | pow / exp / ln / sqrt | `arithmetic` `^`, POWER, GEOMEAN | T-0959, T-1495, the GEOMEAN tasks, T-1524 | Small cell count, but every other kernel calls these primitives. Settle T-1524 before more per-function fits. |
| 14 | weibull / expon | `statistical/weibull.rs`, `exponential.rs` | T-0978 | |
| 15 | bond pricing | `financial/financial_bonds.rs`, TBILL*, DB | T-1475, T-1406 | |
| 16 | normal PDF | `special.rs` `normal_pdf`, `standard_normal_pdf` | T-0975 | All ≤ 4 ulp. |
| 17 | hypergeometric / combinatorial | statrs `Hypergeometric`; MULTINOMIAL | T-0977, T-0927 | |
| 18 | complex | `functions/engineering/complex.rs` | T-0907 | |
| 19 | inverse trig | ASIN/ACOS/ATAN | T-1259 | |

The "(no numeric kernel)" row covers non-kernel numeric misses: array-context SUMPRODUCT
(probes/array-context-legacy-sumproduct), regression functions (LINEST, LOGEST, TREND, GROWTH,
CORREL family) and a few text-length cells. They belong to their own function tasks, not to
this track.

## New tasks

None. Every kernel already has an owner task (columns above). Each owner task's Context now
points at this map.

## Recommended order (decision 0056)

1. T-1524 (x87 model for pow/exp/ln/erfc/gamma). It cuts across ranks 2, 8, 11, 13 and 16, so it
   comes before any of their fits.
2. Rank 1, annuity (T-0901): engine code, mostly evaluation order; the fit lab can enumerate the
   algebraic forms.
3. Rank 4, solvers (T-1525, then T-0916), after rank 1.
4. Rank 2 (T-0968/T-1186), then rank 8 (T-0973), then our own incomplete beta and gamma kernels
   (ranks 3 and 7: T-0970/T-0969, T-0971), then their inverses (ranks 5, 10 and 11).
5. The rest by cell count.
