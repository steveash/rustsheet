---
id: T-2080
title: "CORREL, PEARSON, RSQ: probe inexact constant series and overflow-scale data"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CORREL, PEARSON and RSQ match Excel for a constant series whose mean is inexact and for data whose squares overflow.

## Context
Since T-1723, `PairedMoments::correlation` (crates/rustsheet-engine/src/functions/statistical/correl.rs) is a two-pass
quotient fitted to the regression-pairs and GROUPBY oracle cells, but its only zero-variance evidence is an integer-constant series (probes/regression-pairs-a
column G), which is exact. For {0.1,0.1,0.1} against {1,2,3}, the two-pass mean leaves deviations near 1E-17, so the engine
may return a finite value where Excel may say `#DIV/0!`. Data near 1E200 overflow dx*dx to inf, where Excel may give `#NUM!`. Neither is probed (statistics.md, CORREL/PEARSON/RSQ entries). Probe with the excel-probe skill: those inputs, a few more inexact constants (0.3, 1/3 typed as 0.333..., 1.1),
and 1E154/1E200-scale pairs, for CORREL, PEARSON and RSQ.

## Acceptance criteria
- [x] A filled probe case covers the inputs above; the engine matches every cell, or the entry records what stays open.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/correl-inexact-constant-overflow` (79 cells, excel-graph). Inexact constant series already matched. Overflowing sums of squares are `#NUM!` in Excel (checked before the zero-variance `#DIV/0!`, after the pair count) for CORREL, PEARSON, RSQ and, on x's sum, SLOPE, INTERCEPT, STEYX, FORECAST; `PairedMoments` now returns the error kind and all 79 cells pass. Not a numeric fit (a structural overflow rule, no tuned constants). Open: y-only overflow for STEYX.
