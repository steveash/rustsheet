---
id: T-3060
title: "GAMMA.DIST cumulative at a shape below 1 and a tiny x: Excel's EXP(a LN x)/EXP(GAMMALN(a+1)) leading factor (35 visibly-off t3012-gamma-dist-at-quantile cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
GAMMA.DIST(x,a,beta,TRUE) at a shape below 1 and a small x / beta gives Excel's value to 15 digits where Excel's
own leading factor x^a / Gamma(a+1) is up to 1.2E-14 relative off the exact value. CHISQ.DIST, CHIDIST, POISSON and
the other users of the shared lower series keep every cell they pass today.

## Visible-parity stage
`visible`: Excel is the inaccurate one here. Ours is within 1 to 5 ulp of the exact cdf (mpmath, 50 digits) on
every cell below, and Excel is up to 1.18E-14 relative away, so the fix reproduces Excel's form. It is not an
accuracy fix. Target: the 35 visibly-off cells of probes/t3012-gamma-dist-at-quantile listed below, each within half
a unit of the 15th significant digit of Excel's value. Time-boxed (decision 0056).

## Context
- Gap triage 2026-10-28 (opus queue refill, origin/main 3d4b480b): these 35 cells had no open owner.
  `scripts/exit_owners.py --all-off` listed them under GAMMA.DIST. T-3012 made the probe. T-3013 owns GAMMA.INV in
  the companion case, probes/t3012-gamma-inv-small-shape. Excel's GAMMA.INV is not the crossing of its GAMMA.DIST
  (statistics.md, T-3012 bullet), so T-3013 will not move these cells.
- Case probes/t3012-gamma-dist-at-quantile.case.json, sheet S (excel-graph). Visibly-off cells:
  A3 A4 A5 A16 A17 A18 A21 A24, B3 B4 B5 B16 B17 B18 B21 B24, C3 C4 C5 C16 C17 C18 C21 C24, and
  D3 D4 D5 D16 D17 D18 D20 D21 D22 D23 D25. The D cells are `GAMMA.DIST(..)/1E-k-1` of the B column, so they
  follow B.
- Excel vs ours vs exact (mpmath regularized lower gamma at the cell's x):

  | cell | x | a | Excel | ours | exact | Excel rel | ours rel |
  |---|---|---|---|---|---|---|---|
  | A16 | 4.147567013992206E-134 | 0.75 | 1.000000000000026E-100 | 1.0000000000000131E-100 | 1.0000000000000142E-100 | +1.18E-14 | -1.1E-15 |
  | A17 | 8.935662254176514E-41 | 0.75 | 9.999999999999968E-31 | 9.999999999999929E-31 | 9.9999999999999321E-31 | +3.5E-15 | -3.1E-16 |
  | A18 | 4.1475670139921063E-22 | 0.75 | 9.999999999999981E-17 | 9.99999999999995E-17 | 9.9999999999999616E-17 | +2.0E-15 | -1.1E-15 |
  | A21 | 7.414417701833754E-112 | 0.9 | 1.0000000000000019E-100 | 9.999999999999905E-101 | 9.9999999999999047E-101 | +1.15E-14 | +3.7E-17 |
  | A24 | 7.414417701862728E-12 | 0.9 | 9.999999999999963E-11 | 9.999999999999999E-11 | 1.0000000000000005E-10 | -4.2E-15 | -5.9E-16 |
  | A3 | 6.073048362407712E-301 | 0.1 | 9.999999999999915E-31 | 9.999999999999934E-31 | 9.9999999999999336E-31 | -1.9E-15 | +7.0E-17 |
  | A4 | 6.073048362407875E-161 | 0.1 | 9.999999999999999E-17 | 9.999999999999979E-17 | 9.9999999999999783E-17 | +2.0E-15 | +5.3E-17 |
  | A5 | 6.073048362407845E-101 | 0.1 | 9.999999999999995E-11 | 9.999999999999981E-11 | 9.999999999999981E-11 | +1.4E-15 | -5.7E-18 |

- Fit so far (triage, CPython doubles; Python's `lgamma` and `exp` stand in for Excel's GAMMALN and EXP). Take
  `EXP(a*LN(x))/EXP(GAMMALN(a+1))`, times `EXP(-x)` times the lower series sum (`lower_series`). That is the form
  statistics.md's T-3012 bullet names for a = 0.25. Results, in ulps of Excel's value:
  - It lands every visibly-off A..C cell within 3 ulp: A16, A17 and A18 exactly (ours -58, -17, -14), A21 +0.6
    (ours -51), A3/A4/A5 +2.3 to +2.8 (ours +8.7, -8.9, -6.4), B24/C24 +1.7.
  - It is worse than today on cells that pass at 15 digits now: A23 (a = 0.9, x = 1.6E-18) +32 against ours -2.2,
    A20 (a = 0.75, x = 1.9E-7) +7.6, A24 and A25 (a = 0.9) +17. These cells decide the fit. Possible causes: the
    product a·LN(x) is kept in x87 extended precision, Excel's EXP differs from libm, the GAMMALN, or the form changes
    with x or a. A24 is both a target (Excel 4.2E-15 below exact) and a cell the form misses, so the fit has to
    explain it; if it cannot, the Outcome names A24, B24 and C24 as still open.
  - The power form `x^a/GAMMA(a+1)` never matches (2 ulps above at 1E-65, statistics.md).
- Engine: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs, `incomplete_gamma_pair` →
  `gamma_prefactor` (a < `DIRECT_SHAPE_MAX`: `x.powf(a) * exp(-x) / tgamma(a)`) times `lower_series(a, x)` / a.
  For a < `SMALL_SHAPE` and x < 1 the pair also takes `small_shape_upper`.
  - Excel-faithful pieces already exist: `excel_exp`/`excel_exp_extended` (src/x87_exp.rs), `excel_ln`
    (src/x87_log10.rs), `excel_gammaln` (functions/statistical/gammaln.rs).
  - T-0971 chose `powf` over the log form for GAMMA.DIST(10,9,2,TRUE): the log form was 11 ulps off there. So the
    Excel form has to be limited to the region the evidence supports, perhaps a < 1 and small x.
- Evidence: the case above (rows 2-35, including the passing ones) and statistics.md "Excel's CHISQ.INV and
  GAMMA.INV are exact in the lower tail" (T-3012 evidence). Also cross-check the GAMMA.DIST, CHISQ.DIST, CHIDIST and
  POISSON cells in gen-m9 and ironcalc-calc that pass today. A shape-below-1 point, if the new region reaches it,
  must not regress.
- Needs an oracle sweep only if those cells cannot pin the region: then file a research-lane sweep (excel-probe
  case: GAMMA.DIST(x,a,1,TRUE) at a in {0.1, 0.5, 0.9, 0.99, 1.5} and x from 1E-300 to 1, plus the matching
  `EXP(a*LN(x))/EXP(GAMMALN(a+1))` cells) and set this task blocked on it. Do not fit blind.
- Collisions: T-3013 (GAMMA.INV at a shape below 1) reads the same lower-tail cdf through `quantile_tail`. Land
  this first or coordinate there. T-2852 (GAMMA.DIST density) and T-2638 (tails and densities) touch
  `gamma_density` and the continued fraction, not the lower series' prefactor.

## Acceptance criteria
- [ ] The 35 cells listed in Context are within half a unit of the 15th significant digit of Excel's value (the
      visibly-off count of probes/t3012-gamma-dist-at-quantile in `make eval`'s report is 0).
- [ ] Those cells' `F` lines disappear from eval/baseline/probes.txt (or remain only as within-15-digit ulp lines),
      with no new known-failure line in any suite (`make eval-baseline` refuses regressions).
- [ ] The form and its region are recorded in docs/excel-semantics/statistics.md citing the probe cells, next to
      the T-3012 bullet.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA.INV (T-3013, T-0972, T-2982).
- Bit-level residue left within 15 digits: file it as a `defer: phase-1b` follow-up.
- The density (`cumulative` FALSE): T-2852, T-2638.
