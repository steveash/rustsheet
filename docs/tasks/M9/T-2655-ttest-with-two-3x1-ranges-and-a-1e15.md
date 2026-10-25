---
id: T-2655
title: "TTEST with two 3x1 ranges and a 1E15 tails argument is #VALUE! in Excel, ours #NUM! (gen-m9 ede0323c389fc703 A75)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TTEST of two 3x1 ranges with a huge tails argument is #VALUE!, as in Excel.

## Context
gen-m9/ede0323c389fc703 Calc!A75 (`TTEST` over `Data!B10:B12`, tails 1E15): Excel `#VALUE!`, ours `#NUM!`. Triaged into T-2431 by mistake: both arguments are 3x1, so it is argument validation, not pairing. Check the tails argument handling in functions/statistical/ (t_test).

## Acceptance criteria
- [ ] gen-m9/ede0323c389fc703 Calc!A75 leaves eval/baseline/gen-m9.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
