---
id: T-2865
title: "Probe the M6 lookup argument kinds no Excel cell covers yet: empty text and an error in OFFSET, XLOOKUP, XMATCH, ADDRESS, CHOOSECOLS, CHOOSEROWS, TRIMRANGE"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M6 lookup functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool
(tools/src/rustsheet_tools/gen/generator.py `NUMBER_POOL`) has no empty text or error literal (T-2866), so
generated suites never reach most of these. `array`-mode parameters are not listed: their coverage is not
classified by kind.
These M6 lookup parameters have no Excel cell of the listed kinds in any suite (GETPIVOTDATA is left out: its
arguments need a pivot table, T-2403's area):
- OFFSET.rows, OFFSET.cols, OFFSET.height, OFFSET.width: empty text, error.
- CHOOSE.index_num: empty text.
- INDIRECT.ref_text, INDIRECT.a1: error.
- XLOOKUP.if_not_found: error; XLOOKUP.match_mode, XLOOKUP.search_mode: empty text, error.
- XMATCH.match_mode, XMATCH.search_mode: empty text, error.
- ADDRESS.row_num, ADDRESS.column_num: empty text, error; ADDRESS.abs_num: numeric text, empty text, error;
  ADDRESS.a1: empty text, error, range; ADDRESS.sheet_text: negative, error, range.
- CHOOSECOLS.col_num, CHOOSEROWS.row_num: empty text, error.
- TRIMRANGE.trim_rows: empty text, error; TRIMRANGE.trim_cols: numeric text, empty text, error.
- HYPERLINK.link_location: zero, error; HYPERLINK.friendly_name: error.

Write one probe case, probes/m6-lookup-argument-kind-gaps (excel-probe skill, excel-graph, tag M6), with each kind in two forms where
they differ: a literal (`""`, `NA()`, `#DIV/0!`) and a cell holding it (a formula cell `=""`, `=NA()`). Give every
other argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued
cell in `ERROR.TYPE` too.

## Acceptance criteria
- [ ] probes/m6-lookup-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/lookup.md citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INDEX, MATCH, VLOOKUP, HLOOKUP (M2: T-2861); `array`-mode parameters; generator pools (T-2866).
