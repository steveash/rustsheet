---
id: T-1278
title: "Probe ISBLANK, ISNA, ISTEXT, ISLOGICAL, ISNONTEXT and TYPE on scalar value kinds (evidence gap: 15-23 Excel cells each)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ISBLANK, ISNA, ISTEXT, ISLOGICAL, ISNONTEXT and TYPE each have at least 25 Excel-valued cells in eval, covering every scalar value kind, and any engine mismatch the probe finds is fixed (if a one-line change in the same function) or filed as a task.

## Context
Gap triage 2026-10-09 (decision 0031): `make eval` report.md "Excel evidence per function" lists these M5 functions below the 25-cell threshold: ISBLANK 22 cells (prior 1.5), ISNA 23 (prior 1), ISTEXT 22 (0.5), TYPE 23 (0.2), ISLOGICAL 15 (0.1), ISNONTEXT 15 (0.1). Every eval cell for them passes, but the existing probes only cover range/array lifting (probes/is-functions-array-lift), rich errors (probes/rich-errors-args, error-literals-new) and argument modes (probes/info-logic-argument-modes; information.md "Information function argument modes"). Scalar value kinds are not probed.

Write one probe case `eval/suites/probes/cases/is-functions-scalar-kinds.case.json` (excel-probe skill), with a `Data` sheet holding: a truly blank cell, a cell with a formula returning `""`, a cell with a single space, `0`, `FALSE`, the text `"TRUE"`, the numeric text `"1"`, a date-formatted number, `#N/A`, `#DIV/0!`, `#VALUE!` and a formula returning `NA()`. Then, for each of the six functions, one cell per Data cell (direct reference) plus these direct arguments: `ISBLANK("")`, `ISBLANK(Data!Z99)` (unused cell), `ISBLANK(IF(1,Data!A1))` (blank through IF), `ISNA(#N/A)`, `ISNA(NA()+1)`, `ISNA(#DIV/0!)`, `ISTEXT("")`, `ISTEXT(1&"")`, `ISLOGICAL(1=1)`, `ISLOGICAL("FALSE")`, `ISNONTEXT(Data!<blank>)`, `ISNONTEXT(#N/A)`, `TYPE("")`, `TYPE(TRUE)`, `TYPE(#N/A)`, `TYPE(Data!<blank>)`, `TYPE(LAMBDA(x,x))`. Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. If excel-graph refuses a formula (FileCorruptTryRepair), drop that cell and note it.

Code: `crates/rustsheet-engine/src/functions/information/mod.rs` (`fn_isblank` and the IS* helpers, `fn_type`).

## Acceptance criteria
- [x] `probes/is-functions-scalar-kinds` is committed with its excel-graph oracle sidecar, and report.md's evidence table shows each of the six functions with 25 or more Excel cells (or no longer lists it as below 25).
- [x] docs/excel-semantics/information.md has an entry for the scalar-kind results (VERIFIED, citing the probe cells); anything surprising (for example how `""` from a formula, `TYPE(LAMBDA(..))` or a blank via IF are classed) is called out.
- [x] Every failing probe cell is either fixed in this PR (only if the fix is local to the function) or filed as a new task naming the cells; `make eval-baseline` shows no known failures outside the new case.
- [x] `make check` green; reviewers approve.

## Out of scope
- Range and array lifting (probes/is-functions-array-lift) and the newer rich errors (information.md), already evidenced.
- ISERR/ISERROR/ISNUMBER/ISREF/ISEVEN/ISODD, which already have 25+ Excel cells.

## Outcome
Added probes/is-functions-scalar-kinds (89 excel-graph cells). 88 passed at once; `TYPE(LAMBDA(x,x))`
was `#N/IMPL!` where Excel gives 128, fixed in `fn_type`. All six functions now exceed 25 Excel
cells. Semantics entry added to information.md.
