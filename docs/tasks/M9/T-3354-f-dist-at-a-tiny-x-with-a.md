---
id: T-3354
title: "F.DIST at a tiny x with a huge d2 is off 5E-4"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
F.DIST at x ≈ 1e-12 with d2 ≈ 7.8e6 returns Excel's value instead of one 5e-4 off.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 7). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_f_dist` in crates/rustsheet-engine/src/functions/statistical/fisher.rs, f_density.rs (`excel_f_density`,
  `STIRLING_ERROR` past df 56, an extrapolated fit of decision 0188's code audit) and the incomplete beta for the cdf.
  Find whether the density, the cdf or both are off; at a huge d2 the F law approaches a scaled chi-squared, which
  gives a reference.
- Related: T-2647 (last digits where density·x is subnormal), T-2642 (last ulp at a huge df), T-3364 (the extrapolated
  fits fall back to accurate math). This task is the 5e-4 error, not their last digits.

## Acceptance criteria
- [x] F.DIST (density and cdf) and F.DIST.RT at x from 1e-15 to 1e-6 with d2 from 1e5 to 1e10 match Excel's value
      within 15 significant digits, with the right type and error, on the oracle-filled cells. The Gate A envelope below
      is only a floor on the fresh sweep; Excel's value is the target where it differs from the math by less than
      G = 1e-4.
      Met with the right type and error everywhere; the 15th-digit residue on the cdf and density is T-3467's
      (`defer: phase-1b`, decision 0188 R5).
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [x] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule). Met except the 15th-digit residue (222 cells, all inside Gate A), moved to T-3467
      (`defer: phase-1b`, decision 0188 R5).
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [x] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
Not reproducible on main (2026-10-31): F.DIST has no 5e-4 error at a tiny x with a huge d2. No engine change.
- **Probe:** probes/t3354-f-dist-tiny-x-huge-d2 (excel-graph, 720 cells: F.DIST cdf, density and F.DIST.RT over
  d1 1, 2, 4, 10 and 40, d2 1000 to 1E10 and x 1E-15 to 0.001, the reported region and points on both sides).
  - Every cell is within 1.2e-13 relative of Excel's value (worst A193) and within 1e-9 of the 50-digit math (the
    engine's worst is 1.01e-13 against the math, at A217; Excel's is 1.46e-13, at A193), with the right type. No cell is
    an error on either side.
  - 222 cells (F.DIST cdf and density, none of F.DIST.RT) are beyond half a unit in Excel's 15th digit: Excel's own
    error against the math is up to 1.5e-13 there. They are inside Gate A, so under decision 0188 R5 they move to
    T-3467 (`defer: phase-1b`). Criteria 1 and 3 are met except for that residue, which their
    lines name.
- **Engine-only scans** (no oracle, against mpmath): 1,200 random points over x 1E-15 to 1E-6, d1 1 to 1E7,
  d2 1E5 to 1E10, and 3,000 dense points at x 1E-13 to 1E-11, d2 3E6 to 1.6E7, with integer and non-integer d1 and d2
  (Excel truncates them). The worst error against the math away from underflow is 8e-13, at a result of 6.6E-300.
- **Fresh sweep** (decision 0188 R2, R4.2): probes-numeric/t3354-f-dist-sweep, 200 stratified points times F.DIST cdf,
  density and F.DIST.RT, drawn by docs/research/t3354/sweep.py (seed 3354), filled after the scans and scored by
  docs/research/t3354/score.py:

  | stratum | points | within 15 digits of Excel | within 1e-9 of the math | outside both | error vs number |
  |---|---|---|---|---|---|
  | rows 1-80, the reported region | 240 | 148 | 240 | 0 | 0 |
  | rows 81-140, the acceptance range | 180 | 116 | 180 | 0 | 0 |
  | rows 141-180, extreme but valid | 120 | 109 | 106 | 0 | 0 |
  | rows 181-200, log-uniform | 60 | 50 | 60 | 0 | 0 |

  The 14 extreme points without a math value are ones mpmath could not converge on within 30 s; all 14 are within 15
  digits of Excel, so none is outside the envelope. The 177 sweep cells beyond Excel's 15th digit are also T-3467's.
- **Probable cause:** the incomplete-beta and F density work merged on 2026-10-30 (T-2508 bfrac for b0 ≥ 40, T-2510,
  T-2219, T-3034), after the build the assessment measured. The assessment's raw points are not committed, so this
  cannot be confirmed point for point.
- **Eval delta:** probes known failures 2300 -> 2727 (fixed 0, new-coverage 427, regressions 0, one new case). The
  visible-parity lock moves from 2429 to 2489 (`make eval-baseline`, decision 0093): 2489 cells are visibly off, 222 of
  them in the new case; the other 2267 are 162 below the old lock, cells main fixed since it was last written. No other suite changes.
- **Not a fit:** no engine change, so decision 0188 R4's hold-out and `FITTED` rules do not apply.
- Follow-up: T-3467 (Phase 1b), the 15th-digit residue in both cases.
