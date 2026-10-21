---
id: T-1749
title: "DEVSQ accumulation order: last-digit and cancellation mismatches (probes/stat-order-floor-batch)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
DEVSQ matches Excel's value to the last digit, including under cancellation.

## Context
Oracle: `DEVSQ(L1:L6)` = 48.833333333333336 (ours ...34), `DEVSQ(0.1,0.2,0.3)` = 0.019999999999999997 (ours ...976), `DEVSQ(1E15,1E15+1)` = 0.5 (ours 0), plus 1-ulp cases A278, A280, A290, A300 in probes/stat-order-floor-batch. Excel looks like a mean-then-squared-deviation two-pass; fit the accumulation order against these cells (see coercion.md note on I392).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 1 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on DEVSQ 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 line in eval/baseline/probes.txt disappears (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A277, A278, A280, A290, A294, A295, A300 pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMSQ and other functions (probe `SUMSQ(1,)` before changing it).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (DEVSQ(1E15,1E15+1) is 0, Excel 0.5), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).
