---
id: T-1751
title: "MDETERM of a singular 3x3 matrix: reproduce Excel's 6.66e-16 residual"
milestone: M9
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MDETERM(G1:I3) of the singular matrix [1,2,3;4,5,6;7,8,9] gives Excel's 6.661338147750939e-16 instead of 0.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q107 (excel-graph). Excel's elimination order leaves a rounding residual; ours gives exactly 0. Needs the pivoting/elimination order that reproduces it (numeric fitting, so opus). A second data point: `MDETERM({1,2,3;4,5,6;7,8,10})` is -2.9999999999999982 (S!Q120, which passes today and must keep passing).

## Acceptance criteria
- [x] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [x] The rule is recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The elimination already matched Excel: partial-pivot LU with multiplier-first row updates gives 6.661338147750939E-16
and -2.9999999999999982 bit for bit in IEEE doubles. The mismatch was the `max_val < f64::EPSILON` singularity
cutoff, which returned 0. MDETERM now treats only an exactly-zero pivot column as singular. MINVERSE keeps its
cutoff until T-2063 probes it. arrays.md has the rule and the variants that do not fit. Eval: probes S!Q107
fixed (probes known failures 1793 -> 1792), no regressions.
