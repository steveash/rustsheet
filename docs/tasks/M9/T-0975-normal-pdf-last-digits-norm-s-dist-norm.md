---
id: T-0975
title: "Normal PDF last digits: NORM.S.DIST/NORM.DIST/LOGNORM.DIST non-cumulative and PHI (24 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The normal densities (NORM.S.DIST and NORM.DIST with cumulative FALSE, LOGNORM.DIST FALSE, PHI)
reproduce Excel's last digits from one density helper.

## Context
Kernel map (T-1522, decision 0056): rank 16 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `special::normal_pdf` = `exp(-0.5*d*d) / (SQRT_2PI * std_dev)` with the
correctly rounded constant `SQRT_2PI` = 2.5066282746310007; LOGNORM.DIST's density is inlined in
`log_normal.rs` (`exp(-0.5 d^2) / (x * SQRT_2PI * std_dev)`); PHI (`phi.rs`) uses
`exp(-(x*x)/2) / sqrt(2*PI)`. Three formulations of one concept: consolidate.
24 baseline lines in `eval/baseline/ironcalc-calc.txt`, all NumericUlp:
- `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV`: E3, E5, E35, E46, E63
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.DIST NORM.INV`: E3, E5, E7, E35, E46, E61
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV`: C3, C5, C7, C16, C17, C24, F24, C25, C26, C32, C44, C45
- `ironcalc-calc/STATISTICAL/PHI.xlsx` `Sheet1`: B5

Oracle sweep (T-1041): probes-numeric cases `m9-normpdf-norm-s-dist`, `m9-normpdf-phi`,
`m9-normpdf-norm-dist-s0p5`, `-s1p2`, `-s3`, `-s10`, `m9-normpdf-lognorm-dist`.

Examples (expected vs actual):
- `NORM.S.DIST(0,FALSE)` C7/C25/C45: expected 0.3989422804014327, actual 0.39894228040143265.
- `NORM.S.DIST(1.5,FALSE)` C24 (and C26, C32, C44, F24 `NORM.DIST(1.5,0,1,FALSE)`): expected
  0.12951759566589174, actual 0.12951759566589172.
- `LOGNORM.DIST(4,3.5,1.2,FALSE)` E3: expected 0.01761759668181922, actual 0.017617596681819225.
- `PHI(...)` B5: expected 0.028327037741601186, actual 0.02832703774160119.

Lead found during triage (`docs/excel-semantics/statistics.md`, "Standard normal density uses
sqrt(2*pi) computed in doubles", UNVERIFIED): `exp(-z*z/2) / sqrt(2*pi)` with `sqrt(2*pi)` computed
in doubles (2.5066282746310002), or equivalently multiplying by the correctly rounded 1/sqrt(2*pi),
reproduces all 9 numeric NORM.S.DIST density cells (rows 3, 5, 7, 16, 17, 24, 26, 32, 44). For
NORM.DIST with sigma != 1 the candidates split (`K*exp(-d^2/2)/sigma` matched 4 of 5 checked cells,
the computed-sqrt form 3 of 5) and LOGNORM.DIST matched only the current form (3 of 5), so the
placement of sigma and x in the expression still has to be fitted. Enumerate the few operation
orders (constant vs computed sqrt(2 pi); divide vs multiply by the reciprocal; where sigma and x
enter; `d*d/2` vs `0.5*d*d`) against all cached density cells, including the currently passing ones.

## Acceptance criteria
- [ ] The 12 NORM.S.DIST density lines leave `eval/baseline/ironcalc-calc.txt`; the NORM.DIST,
      LOGNORM.DIST and PHI lines leave too if one formulation per function fits every cached density
      cell, otherwise the PR lists them and they go to the T-1041 normal-PDF sweep. No new
      lines in any baseline.
- [ ] One density helper in `special.rs` used by NORM.S.DIST, NORM.DIST, LOGNORM.DIST and PHI (or a
      comment citing cells that prove a function needs a different order).
- [ ] The statistics.md entry is updated to VERIFIED or rewritten from the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
Cumulative normal / erfc (T-0968); inverse normal (T-0925).
