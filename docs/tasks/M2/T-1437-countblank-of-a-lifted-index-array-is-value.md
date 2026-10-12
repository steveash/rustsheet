---
id: T-1437
title: "COUNTBLANK of a lifted INDEX array is #VALUE! per element; COUNTA counts blank elements of a lifted INDEX"
milestone: M2
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`COUNTBLANK(INDEX(D1:D5,{1,2,3}))` is `#VALUE!` in each of the three spilled cells and `COUNTA(INDEX(D1:D5,{1,2,3}))` is 3 (blank
D2 counts), as in Excel.

## Context
probes/index-lifted-edge-cases (T-1376, excel-graph) H131:J131 and H136 fail: the engine gives 0 and 2. COUNTBLANK needs a
reference, and a lifted INDEX array is not one, so every element is `#VALUE!` (the formula spills 1x3). COUNTA counts the
blank element of the array (D2 is an empty element of the lifted INDEX result), the same finding as T-1395 (IFERROR array);
do both together or build on T-1395. See docs/excel-semantics/logic.md ("IFERROR / IFNA of a blank cell is an empty value")
and lookup.md ("INDEX lifts over array and range indices"). Code: functions/statistical/count.rs.

## Acceptance criteria
- [ ] H131:J131 and H136 of probes/index-lifted-edge-cases pass; `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The TAKE-of-blank case (T-1395 decides it).
