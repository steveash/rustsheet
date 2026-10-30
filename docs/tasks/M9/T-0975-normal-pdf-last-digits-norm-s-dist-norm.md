---
id: T-0975
title: "Normal PDF last digits: NORM.S.DIST/NORM.DIST/LOGNORM.DIST non-cumulative and PHI (24 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
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
`m9-normpdf-norm-dist-s0p5`, `m9-normpdf-norm-dist-s1p2`, `m9-normpdf-norm-dist-s3`, `m9-normpdf-norm-dist-s10`, `m9-normpdf-lognorm-dist`.
Visibly off in `m9-normpdf-lognorm-dist` on main 5b8c2ae7 (T-3021): Calc!A29 `LOGNORM.DIST(7.25,0,1,FALSE)` Excel
0.007734043882067061, ours ...053; Calc!A41 `LOGNORM.DIST(0.25,1,0.5,FALSE)` Excel 3.6133196398262184E-5, ours ...116;
Calc!A73 `LOGNORM.DIST(8.25,1,0.5,FALSE)` Excel 0.008220277976125383, ours ...373.

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

gen-m9 sweep (gap triage 2026-10-23, T-2255). `LOGNORM.DIST(40,0,0.5,FALSE)` Excel 3.0218137820578166E-14, ours
3.021813782057842E-14 (1 cell, the lognormal density).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/b4173fe25109a238 Calc!A48 `=LOGNORM.DIST(40,0,(Data!A10*100%),FALSE)`: Excel 3.0218137820578166e-14, ours 3.021813782057842e-14
Cells:
- gen-m9/b4173fe25109a238.case.json (1): Calc!A48

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: LOGNORM.DIST density last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/3a25a03eb94fcfbd Calc!A220 `=LOGNORM.DIST((Data!C11+Data!C9),FLOOR.PRECISE((Data!D2^Data!C4)),Data!E9,FALSE())`: Excel 7.694598626706419e-22, ours 7.6945986267064185e-22
Cells:
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A220
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A310
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A250

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): NORMDIST pdf last digits (cumulative FALSE), none visibly off.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 16 known-failure lines in eval/baseline/gen-m9.txt, 0 visibly off.
Examples:
- gen-m9/1b31092a0d79e5f1 Calc!A157 `=NORMDIST(100,SUM((Data!$B4-2)),1E+10,FALSE)`: Excel 3.989422804014327e-11, ours 3.9894228040143264e-11
Cells:
- gen-m9/097af1ca33638670.case.json (1): Calc!A236
- gen-m9/11eb3ab4f4f8104c.case.json (2): Calc!A223, A285
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A157
- gen-m9/334f3dc9ca937f10.case.json (2): Calc!A321, A383
- gen-m9/3b7990bd4e3307ba.case.json (2): Calc!A153, A277
- gen-m9/73f053591a54bc43.case.json (2): Calc!A109, A126
- gen-m9/787ba8d801105a60.case.json (1): Calc!A261
- gen-m9/80586b859b73c87d.case.json (3): Calc!A199, A318, A335
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A177
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A239

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 17 known-failure lines (1 from T-2255, 16 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
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

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: gen-m9/b4173fe25109a238 Calc!A48 `=LOGNORM.DIST(40,0,(Data!A10*100%),FALSE)`: Excel 3.0218137820578166e-14, ours 3.021813782057842e-14 (5 units of the 15th digit).

## Phase 1 triage (T-0090, 2026-10-27)
It owns the 4 visibly-off LOGNORM.DIST cells and alone lifts LOGNORM.DIST. Priority raised so it leads its stage in M9.
