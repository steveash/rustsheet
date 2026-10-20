---
id: T-1724
title: "COVAR skips pairs with text, blanks or booleans and accepts scalars as Excel (T-1620 probes)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COVAR matches Excel for ranges holding text, blanks or booleans, and for scalar arguments.

## Context
Found by T-1620 (probes/regression-pairs-a/-b, regression-pairs-b column L; excel-graph 2026-10-16). Excel skips a pair when either cell is text, blank or boolean: `COVAR(A1:A6,C1:C6)` (C2 text) 2.6399999999999997, `COVAR(D1:D6,B1:B6)` (D2 blank) 2.52, `COVAR(E1:E6,B1:B6)` (E2 TRUE) 2.52, `COVAR(J1:J6,B1:B6)` 3.44; ours is `#N/A` for each. Scalar arguments: `COVAR(5,5)` and `COVAR(A1,B1)` are 0 (ours `#VALUE!`), `COVAR(A1:A6,"a")` `#N/A` (ours `#VALUE!`). Check COVARIANCE.P and COVARIANCE.S for the same (probe them if absent) and record in docs/excel-semantics/statistics.md. See also T-1292.

The scalar part overlaps T-1401. COVAR with every pair skipped (`COVAR({1,2,3},{TRUE,FALSE,TRUE})`) is `#DIV/0!` although one pair gives 0.

Reuse `PairedMoments` (statistical/correl.rs, T-1723: two-pass sums about the means) for COVAR and COVARIANCE.P/S
(Sxy/n, Sxy/(n-1)) rather than their own sums, if it reproduces the probed bits.

## Acceptance criteria
- [ ] The mismatching non-ULP COVAR cells of probes/regression-pairs-b pass and the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing further inputs (file a probe task instead).

## Outcome
COVAR, COVARIANCE.P and COVARIANCE.S skip pairs with text, blanks or booleans (the count-mismatch `#N/A` is gone) and a typed text sample is `#N/A`. New probe probes/covariance-pairs-skip (excel-graph) covers P and S. Scalar `COVAR(5,5)` already passed. Eval: probes known failures shrank (see PR).
