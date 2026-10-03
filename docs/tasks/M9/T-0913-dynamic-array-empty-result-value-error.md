---
id: T-0913
title: "DROP/TAKE/FILTER/SEQUENCE: empty or invalid dynamic-array results are #VALUE!, not #CALC!"
milestone: M9
lane: functions
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
Where Excel returns `#VALUE!` for an array function whose result would be empty or whose arguments are out of range, the engine returns `#CALC!` today. Make the engine match the Excel-cached values.

## Context
Evidence is Excel-cached values in the upstream workbooks (no oracle needed). Failing cells, all ErrorMismatch (expected `#VALUE!`, actual `#CALC!`):
- `ironcalc-calc/LOOKUP_AND_REFERENCE/DROP_TAKE.xlsx`: Sheet1!C2 `DROP(A2:A8,-7)`, E2 `DROP(A2:A8,1200)`, F2 `DROP(A2:A8,7)`, I2 `DROP(A2:A8,1,1)`, L2 `TAKE($A$2:$A$8,0)`.
- `ironcalc-calc/LOOKUP_AND_REFERENCE/FILTER_ERRORS.xlsx`: Sheet1!E1 `FILTER(A1:A4,A1:A4>10)` (no matches).
- `ironcalc-calc/MATH_AND_TRIGONOMETRY/SEQUENCE.xlsx`: Sheet1!C16 `SEQUENCE(0,0)`, E18 `SEQUENCE(0.3)`.
Suspected root cause: a shared "empty array" path emits `Error::CALC`; the cached Excel file shows `#VALUE!` (the xlsx cached-value reader may also surface `#CALC!` differently; check `rustsheet-eval show` output for these cells). Search the engine for `Error::CALC` in the DROP/TAKE/FILTER/SEQUENCE implementations and in the cached-error decoding. Do not change other functions where the evidence shows `#CALC!` (e.g. FILTER with an if_empty-less empty result in other cases; check every other `#CALC!` expectation in eval/ first with `rg '#CALC'`).

## Acceptance criteria
- [ ] The 8 cells above no longer appear as F lines in eval/baseline/ironcalc-calc.txt, and no new F lines are added.
- [ ] Unit tests cover each of DROP, TAKE, FILTER, SEQUENCE returning `#VALUE!` for the empty case.
- [ ] `make check` green.

## Out of scope
SEQUENCE numeric ULP cells (I9..M14) in the same workbook.
