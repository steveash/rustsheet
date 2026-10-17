---
id: T-1749
title: "DEVSQ accumulation order: last-digit and cancellation mismatches (probes/stat-order-floor-batch)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
DEVSQ matches Excel's value to the last digit, including under cancellation.

## Context
Oracle: `DEVSQ(L1:L6)` = 48.833333333333336 (ours ...34), `DEVSQ(0.1,0.2,0.3)` = 0.019999999999999997 (ours ...976), `DEVSQ(1E15,1E15+1)` = 0.5 (ours 0), plus 1-ulp cases A278, A280, A290, A300 in probes/stat-order-floor-batch. Excel looks like a mean-then-squared-deviation two-pass; fit the accumulation order against these cells (see coercion.md note on I392).

## Acceptance criteria
- [ ] A277, A278, A280, A290, A294, A295, A300 pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMSQ and other functions (probe `SUMSQ(1,)` before changing it).
