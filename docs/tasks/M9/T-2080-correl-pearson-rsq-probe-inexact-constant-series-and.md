---
id: T-2080
title: "CORREL, PEARSON, RSQ: probe inexact constant series and overflow-scale data"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CORREL, PEARSON and RSQ match Excel for a constant series whose mean is inexact and for data whose squares overflow.

## Context
Since T-1707, `excel_correlation` (crates/rustsheet-engine/src/functions/statistical/pearson.rs) is a two-pass quotient
fitted to 44 oracle cells, but its only zero-variance evidence is an integer-constant series (probes/regression-pairs-a
column G), which is exact. For {0.1,0.1,0.1} against {1,2,3}, the two-pass mean leaves deviations near 1E-17, and the engine
gives 0, not `#DIV/0!` ({0.7}x6 gives 0 too; {1.1}x5 gives `#DIV/0!`). Data near 1E200 overflow dx*dx to inf; the engine then
gives `#DIV/0!`, where Excel might give `#NUM!`. Neither is probed (statistics.md "CORREL, PEARSON and RSQ ... their last digit",
open points). Probe with the excel-probe skill: those inputs, a few more inexact constants (0.3, 1/3 typed as 0.333..., 1.1),
and 1E154/1E200-scale pairs, for CORREL, PEARSON and RSQ.

## Acceptance criteria
- [ ] A filled probe case covers the inputs above; the engine matches every cell, or the entry records what stays open.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
