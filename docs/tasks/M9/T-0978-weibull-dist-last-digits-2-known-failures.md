---
id: T-0978
title: "WEIBULL.DIST last digits (2 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1046]
---

## Goal
WEIBULL.DIST (cumulative and density) reproduces Excel's last digits.

## Context
Kernel map (T-1522, decision 0056): rank 14 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric case `m9-weibull-dist` (T-1046).
Triage T-0903. Kernel: `statrs::distribution::Weibull` (`cdf` = `-expm1(-x^k * scale^-k)`,
`pdf` = `k (x/l)^(k-1) exp(-x^k scale^-k) / l`), called from `functions/statistical/weibull.rs`.
2 baseline lines in `eval/baseline/ironcalc-calc.txt`, both NumericUlp:
- `ironcalc-calc/STATISTICAL/WEIBULL_DIST.xlsx` `Sheet1`: E2 `WEIBULL.DIST(0.8,1,9,TRUE)`: expected
  0.08505277126996899, actual 0.085052771269969; E3 `WEIBULL.DIST(0.4,1,9,FALSE)`: expected
  0.10628097101144768, actual 0.10628097101144769.
Triage lead (two cells only, so not a fit): for E2, `-expm1(-exp(k*(ln x - ln l)))` gives
0.08505277126996899 (match), while `1 - exp(-(x/l)^k)` and the current form do not; no tried density
form (`k/l (x/l)^(k-1) exp(-(x/l)^k)`, `exp(-t)/l`, the log-space form) gives E3.
**Needs an oracle sweep (T-0018):** fit against the T-1046 Weibull sweep, never blind.

## Acceptance criteria
- [ ] Both lines leave `eval/baseline/ironcalc-calc.txt`, the T-1046 Weibull probe cells pass (or the
      PR lists the residue), no new lines in any baseline.
- [ ] The matched formulation is recorded in `docs/excel-semantics/statistics.md`.
- [ ] `make check` green; reviewers approve.

## Out of scope
EXPON.DIST (done in T-0928).
