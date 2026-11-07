---
id: T-4912
title: "Read a written-empty branch of IF, CHOOSE and IFERROR as an omission in ToolPak functions and the newer roundings (T-4901 follow-up)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
MROUND, QUOTIENT, SERIESSUM and the newer roundings (CEILING.MATH, FLOOR.PRECISE) read an IF, CHOOSE or IFERROR whose
chosen branch is written empty as an omission, as Excel does, while a formula that merely returns a blank stays 0.

## Context
- probes/t4901-mround-blank-let-lambda-roundings (excel-graph, 2026-11-07, T-4901): `MROUND(IF(TRUE,),1)`,
  `MROUND(IF(FALSE,1,),1)`, `MROUND(CHOOSE(1,,2),1)`, `MROUND(IFERROR(,1),1)`, `MROUND(5,IF(TRUE,))` and
  `QUOTIENT(IF(TRUE,),1)` are `#N/A` (A3, A10, A12, A13, A14, A16), and `CEILING.MATH(2.675,IF(TRUE,))` is 3, the
  omitted significance (A15). Ours reads all of them as 0 (`Model::is_empty_argument` looks only at the argument node
  and a LET/LAMBDA variable). A blank from VLOOKUP/INDEX of an empty cell stays 0 (A1, A2, A8, A9), so the
  look-through must be on the written branch, not on `CalcResult::EmptyArg`, which IFERROR(<blank>,5) also gives.
- The condition (or index) of the wrapper is needed to pick the branch, so the predicate needs the cell and a
  `&mut self` evaluation; design that without double evaluation.

## Acceptance criteria
- [ ] A3, A10, A12:A16 of the probe pass; the cells owned by this task leave eval/baseline/probes.txt.
- [ ] Other IF/CHOOSE/IFERROR/IFNA/IFS-style wrappers probed (excel-probe) and the rule recorded in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LET/LAMBDA omissions (already evidenced).
