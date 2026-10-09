---
id: T-1203
title: "RANK with an array (IF/non-reference) ref argument is #VALUE! spilled over the array"
milestone: M1
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`RANK(5,IF(A1:C1>0,A1:C1))` (an array, not a reference, as the ref argument) matches Excel: `#VALUE!`, spilled over the array's shape.

## Context
From T-1152's probe probes/multiarea-order-stats: Sheet1!H14:J14 are `#VALUE!` in Excel (the result spills 1x3 like the IF array); the engine returns 2 in H14. Semantics: docs/excel-semantics/references.md "Order statistics over unions". Probe RANK.AVG and RANK with other non-reference second arguments (constant arrays, INDEX results, CHOOSE) before fixing.

## Acceptance criteria
- [ ] probes/multiarea-order-stats H14:J14 pass, and a probe covers the other non-reference second arguments of RANK and RANK.AVG.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
