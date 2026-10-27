---
id: T-2875
title: "Probe the GROUPBY and PIVOTBY option argument kinds no Excel cell covers yet: empty text, an error, numeric text or a boolean"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every option parameter of GROUPBY and PIVOTBY listed below has at least one Excel-backed cell for each argument
kind the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M7 parameters have no Excel cell of the listed kinds in any suite:
- GROUPBY.field_headers: empty text, an error. GROUPBY.total_depth: numeric text, empty text, a boolean, an error.
  GROUPBY.field_relationship: empty text, an error, a blank cell.
- PIVOTBY.field_headers, PIVOTBY.row_total_depth: numeric text, empty text, an error.
  PIVOTBY.col_total_depth: empty text, a boolean, an error. PIVOTBY.relative_to: text, empty text, an error.
Use a small data block (keys a, b, a, c; values 1..4) and `_xleta.SUM`; read each result through ROWS/COLUMNS and
`TEXTJOIN("|",FALSE,MAP(r,LAMBDA(x,IFERROR(x&"","#"&ERROR.TYPE(x)))))` as the T-2578 probes do, so blank cells and
errors inside the result are scored.

Write one probe case, probes/m7-groupby-argument-kind-gaps (excel-probe skill, excel-graph, tag M7).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m7-groupby-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/arrays.md (GROUPBY and PIVOTBY section) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifted options (T-2451); PIVOTBY's Total gate (T-2606); blank keys (T-2460).
