---
id: T-2228
title: "A subnormal cell constant reads as 0 (LOG10 #NUM!, =0 TRUE), as Excel loads it"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
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

## Acceptance criteria
- [x] probes/subnormal-cell-constants passes in full and leaves the probes baseline (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Subnormal formula results (kept, numbers.md "Worksheet EXP") and their export (T-2211).

## Outcome
The flush is at load, the one place Excel's evidence covers: `rustsheet-xlsx` import passes a `<v>` number constant through
`flush_subnormal` (via `util.rs` `parse_constant_number`; now `pub` and re-exported from the engine, still the one definition), and `rustsheet-eval`'s `spec_input`
does the same for a case's number constants, since `spec2xlsx` writes them as `<v>` and that is what Excel loaded. A spill's
cached value (a formula result) is not flushed, nor is a number set through the API (`Model::update_cell_with_number`,
facade `Input::from`): typing a subnormal is unprobed. The probe now also has the negatives -5E-324 and -1E-310 (both flushed: `=0` TRUE) and -2.2250738585072014E-308 (kept),
refilled by excel-graph 2026-10-21; probes/subnormal-cell-constants passes 35/35 (12 known failures gone);
`make eval-baseline` also recorded poi FormulaEvalTestData_Copy.xlsx EverythingTests!I1360, which already passed on main
without this change. Test: `rustsheet-xlsx/tests/subnormal_constant_import.rs` (round trip: 5e-324 and 1e-310 load as 0,
`=A1=0` TRUE, LOG10 `#NUM!`; the smallest normal and a reloaded `=EXP(-745)` keep their values).
