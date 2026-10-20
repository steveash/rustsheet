---
id: T-2181
title: "SHEETS with a reference argument counts the sheets the reference spans"
milestone: M5
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-2115]
---

## Goal
`SHEETS(reference)` returns the number of sheets the reference spans, as Excel does, instead of `#N/IMPL!`.

## Context
- From T-2115. probes/info-sheets-info-types (excel-graph, 2026-10-20) Alpha!D1:D24 are all `#N/IMPL!` in the engine
  (`fn_sheets`, crates/rustsheet-engine/src/functions/information/mod.rs, returns `#N/IMPL!` for any argument).
- Excel: a cell or range on one sheet is 1 (`SHEETS(A1)`, `SHEETS(Data!A1:B2)`, `SHEETS($A$1:$B$2)`); a 3-D span counts
  its sheets in either order (`SHEETS(Alpha:Gamma!A1)` 3, `SHEETS(Gamma:Alpha!A1)` 3, `SHEETS(Beta:Data!A1:B2)` 3,
  `SHEETS(Alpha:Data!B2)` 4); the result is a number (`TYPE` 1, `+0`, `*1`, `=3` work). docs/excel-semantics/information.md
  "SHEETS counts the sheets a reference spans".
- Read the argument with the existing reference readers (`evaluate_reference_arg` / `get_sheet_span_range`; a 3-D range
  is a `Range` whose corners are on two sheets) and count the sheet positions between the corners, inclusive. Do not
  write a second span resolver (directive 3).
- Unprobed (the oracle canary refuses them, T-2113): a union, INDIRECT, OFFSET, a defined name, a non-reference
  argument (`SHEETS("x")`, `SHEETS(1)`, `SHEETS({1,2})`) and an error argument. Keep today's results for those unless
  evidence appears; do not guess.

## Acceptance criteria
- [ ] probes/info-sheets-info-types Alpha!D1:D24 and probes/sheet-names-like-cells-3d-spans S1!B3, B4, B6 (unquoted
  cell-like sheet names: `#VALUE!`, 1; quoted: 3) pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] Unit tests for a one-sheet reference, a 3-D span in both orders and the bare `SHEETS()` (unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The unprobed argument forms listed above.
