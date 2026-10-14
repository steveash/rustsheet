---
id: T-0977
title: "HYPGEOM.DIST last digits (2 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1048, T-1049]
---

## Goal
HYPGEOM.DIST (density and cumulative) reproduces Excel's last digits.

## Context
Kernel map (T-1522, decision 0056): rank 17 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::Hypergeometric` (`pmf` = ratio of `factorial::binomial`
values; `cdf` = sum of `exp(ln_binomial + ln_binomial - ln_binomial)` terms), called from
`functions/statistical/hypegeom.rs` `fn_hyp_geom_dist`.
2 baseline lines in `eval/baseline/ironcalc-calc.txt`, both NumericClose:
- `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` `Sheet1`: F2 `HYPGEOM.DIST(1.8,4.2,8.2,22,TRUE)`: expected
  0.5349282296650714, actual 0.5349282296650703; F3 `HYPGEOM.DIST(1.8,4.2,8.2,18.34,FALSE)`: expected
  0.31372549019607815, actual 0.3137254901960784.
Excel is *not* the exact rational here (16/51 = 0.3137254901960784, which statrs already returns;
559/1045 = 0.5349282296650718), and neither a log-gamma nor a log-factorial exp-sum form reproduces
both cells (`docs/excel-semantics/statistics.md`, "HYPGEOM.DIST is not exact for small integer
arguments").
**Needs an oracle sweep (T-0018):** two cells cannot pin a formulation; fit against the
HYPGEOM.DIST evidence in T-1049 and HYPGEOMDIST evidence in T-1048, never blind. Candidates to try: pmf
via the binomial recurrence starting from the mode, pmf via `exp` of a Stirling-series / lgamma
difference, CDF as ascending vs descending sums, CDF via the complement when k is above the mean.

## Acceptance criteria
- [ ] Both lines leave `eval/baseline/ironcalc-calc.txt`, the T-1049 HYPGEOM.DIST probe cells pass, no
      new lines in any baseline.
- [ ] Bounded loops for huge populations (no unbounded allocation or iteration on untrusted input).
- [ ] The statistics.md entry is updated with the matched formulation.
- [ ] `make check` green; reviewers approve.

## Out of scope
BINOM/NEGBINOM/POISSON (T-0927).
