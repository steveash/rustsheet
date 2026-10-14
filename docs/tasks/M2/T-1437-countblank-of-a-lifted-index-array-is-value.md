---
id: T-1437
title: "COUNTBLANK of a lifted INDEX array is #VALUE! per element; COUNTA counts blank elements of a lifted INDEX"
milestone: M2
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`COUNTBLANK(INDEX(D1:D5,{1,2,3}))` is `#VALUE!` in each of the three spilled cells and `COUNTA(INDEX(D1:D5,{1,2,3}))` is 3 (blank
D2 counts), as in Excel.

## Context
probes/index-lifted-edge-cases (T-1376, excel-graph) H131:J131 and H136 fail: the engine gives 0 and 2. COUNTBLANK spills a 1x3 of `#VALUE!` (one per element; the likely reason is that a lifted INDEX array is not a reference, unprobed). COUNTA counts the
blank element of the array (D2 is an empty element of the lifted INDEX result), the same finding as T-1395 (IFERROR array);
do both together or build on T-1395. See docs/excel-semantics/logic.md ("IFERROR / IFNA of a blank cell is an empty value")
and lookup.md ("INDEX lifts over array and range indices"). Code: functions/statistical/count.rs.

T-1395 made COUNTA count every element of an array value, so H136 passes; only the COUNTBLANK cells H131:J131 remain.

## Acceptance criteria
- [x] H131:J131 and H136 of probes/index-lifted-edge-cases pass; `make eval-baseline` records the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- The TAKE-of-blank case (T-1395 decides it).

## Outcome
COUNTBLANK of any argument that is a value rather than a reference is `#VALUE!`; an array value lifts it to a
`#VALUE!` array of its shape (an error element keeps its error). New probe probes/countblank-array-values (35 cells,
excel-graph) established the general rule: lifted INDEX shapes, CHOOSE and IF with array conditions, `CHOOSE(1,5)` and
`CHOOSE(1,"")` (the old code counted an empty-string value as blank). Static analysis now lets COUNTBLANK spill
(`reference_parameter.rs`, shared with RANK, plus a CHOOSE-with-array-index case). The array result reuses
`not_a_reference_array` (reference_operators.rs, shared with SUBTOTAL and AREAS).
Eval delta: probes/index-lifted-edge-cases H131:J131 fixed; new case 30/35 passing; 0 regressions. Remaining cells
filed: T-1507 (1x1 array index to INDEX is still a reference, H1) and T-1508 (CHOOSE with an array index over range
arms gives the wrong grid shape, H21/I21, H35/I35).
