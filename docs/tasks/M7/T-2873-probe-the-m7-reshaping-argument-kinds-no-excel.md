---
id: T-2873
title: "Probe the M7 reshaping argument kinds no Excel cell covers yet: empty text and an error in TAKE, DROP, EXPAND, WRAPCOLS, WRAPROWS, TOCOL and TOROW"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M7 reshaping functions below has at least one Excel-backed cell for each argument
kind the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M7 parameters have no Excel cell of the listed kinds in any suite:
- TAKE.rows, TAKE.columns: numeric text, empty text, an error.
- DROP.rows, DROP.columns: empty text, an error.
- EXPAND.rows: numeric text, empty text, an error. EXPAND.columns: zero, empty text, an error.
- WRAPCOLS.wrap_count, WRAPROWS.wrap_count: numeric text, empty text (T-2479 probes errors and bad counts).
- TOCOL.ignore: empty text, an error. TOCOL.scan_by_column: zero, numeric text, empty text, an error.
- TOROW.ignore, TOROW.scan_by_column: empty text, an error, a multi-cell range.
Use A1:C3 = 1..9 as the array. Read each array result through ROWS/COLUMNS and a TEXTJOIN of its cells, so the shape is scored and not only the top-left value.

Write one probe case, probes/m7-reshape-argument-kind-gaps (excel-probe skill, excel-graph, tag M7).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m7-reshape-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/arrays.md citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting over arrays of these arguments (T-2299, T-2519); WRAPCOLS errors and EXPAND pad_with (T-2479).
- SEQUENCE, MAKEARRAY, SORT, SORTBY, UNIQUE (T-2874); GROUPBY and PIVOTBY (T-2875).
