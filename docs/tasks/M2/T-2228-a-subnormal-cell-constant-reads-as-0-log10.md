---
id: T-2228
title: "A subnormal cell constant reads as 0 (LOG10 #NUM!, =0 TRUE), as Excel loads it"
milestone: M2
lane: engine
status: todo
priority: P1
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
A number cell constant in a loaded workbook (xlsx `<v>`) that is a non-zero subnormal (below 2.2250738585072014E-308) evaluates as 0 in formulas, as
Excel for the web reads it: `LOG10(A1)` and `LN(A1)` are `#NUM!`, `A1=0` is TRUE, `A1*1E+300` is 0.

## Context
- Found by T-2016: probes/subnormal-cell-constants (excel-graph, 2026-10-21) rows 1-3 (5e-324, 2.2e-308, 1e-310) against
  row 4 (the smallest normal, kept). `ISNUMBER` stays TRUE, so the cell is a number 0, not a blank or text.
- docs/excel-semantics/numbers.md "A subnormal cell constant reads as 0" (T-2016); the formula-literal case is the
  T-1007 entry ("a subnormal literal argument reads as 0").
- Decide where the flush belongs: at load (xlsx `<v>` and spec-case constants) or where a cell's number is read.
  Reuse `arithmetic::flush_subnormal`. Keep the export side (T-2211, formula results) apart: a subnormal formula
  result such as `EXP(-745)` is kept in memory.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 3 cells, 0 numeric past half a unit in the 15th digit and 3 with a wrong type, error or text, blamed on LOG10 3.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| probes/subnormal-cell-constants Sheet1!B1 | `=LOG10(A1)` | #NUM! | -323.3062153431158 |
| probes/subnormal-cell-constants Sheet1!B2 | `=LOG10(A2)` | #NUM! | -307.6575773191778 |
| probes/subnormal-cell-constants Sheet1!B3 | `=LOG10(A3)` | #NUM! | -310 |

## Acceptance criteria
- [ ] Visible parity: the 3 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 3 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/subnormal-cell-constants passes in full and leaves the probes baseline (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Subnormal formula results (kept, numbers.md "Worksheet EXP") and their export (T-2211).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (LOG10 of a subnormal cell constant is a number where Excel is `#NUM!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.
