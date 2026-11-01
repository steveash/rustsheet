---
id: T-2874
title: "Probe the M7 SEQUENCE, MAKEARRAY, SORT, SORTBY and UNIQUE argument kinds no Excel cell covers yet: empty text, an error or a range"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of SEQUENCE, MAKEARRAY, SORT, SORTBY and UNIQUE listed below has at least one Excel-backed
cell for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M7 parameters have no Excel cell of the listed kinds in any suite:
- SEQUENCE.rows: empty text. SEQUENCE.columns, SEQUENCE.start, SEQUENCE.step: empty text, an error.
- MAKEARRAY.rows: numeric text, empty text, an error. MAKEARRAY.columns: empty text, an error
  (with `LAMBDA(r,c,r*10+c)`).
- SORT.by_col: empty text, an error, a multi-cell range (T-2366 probes lifted by_col elements, not these kinds).
- SORTBY.sort_order: empty text, an error.
- UNIQUE.by_col, UNIQUE.exactly_once: empty text, an error.
Read each array result through ROWS/COLUMNS and a TEXTJOIN of its cells, so the shape is scored and not only the top-left value. RANDARRAY's gaps are left out: it is
volatile, and the harness skips its cells.

Write one probe case, probes/m7-array-argument-kind-gaps (excel-probe skill, excel-graph, tag M7).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m7-array-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/arrays.md citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifted by_col and sort_order elements (T-2366, T-2349, T-2370).
- RANDARRAY (volatile); TAKE, DROP and the other reshaping functions (T-2873).
