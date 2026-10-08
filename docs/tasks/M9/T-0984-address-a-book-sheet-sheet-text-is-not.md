---
id: T-0984
title: "ADDRESS: a [Book]Sheet sheet_text is not quoted (1 poi known failure)"
milestone: M9
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ADDRESS(2,3,3,TRUE,"[Book1]Sheet1")` returns `[Book1]Sheet1!$C2` as Excel does: a leading
`[workbook]` part of `sheet_text` does not by itself make ADDRESS quote the name.

## Context
Found by the gap triager on 2026-10-06. Failing cell: `poi/FormulaEvalTestData_Copy.xlsx`
(Excel-cached, Excel 15) EverythingTests!F128 `=ADDRESS(2,3,3,TRUE,"[Book1]Sheet1")`: expected
`[Book1]Sheet1!$C2`, actual `'[Book1]Sheet1'!$C2`. Passing neighbours D128 `ADDRESS(2,3)` and E128
`ADDRESS(2,3,2)`, and upstream ironcalc-calc LOOKUP_AND_REFERENCE/ADDRESS.xlsx Sheet1!E2 (`"Sheet2"`
unquoted), B33 (`"A Wird One$"` quoted) and B38 (`"Me lo invente"` quoted) must stay green.
Semantics: docs/excel-semantics/references.md "ADDRESS leaves a [Book]Sheet name unquoted".

Code: `fn_address` in `crates/rustsheet-engine/src/functions/lookup_and_reference/address_areas.rs`
builds the prefix with `quote_name` (`crates/rustsheet-engine/src/expressions/utils/mod.rs`), which
quotes any name holding a character outside letters, digits, `_` and `.`, so `[` and `]` force
quotes. `quote_name` also serves formula stringification, so do not change it: in ADDRESS, when
`sheet_text` starts with a `[...]` part, apply the quoting decision to the text after the `]` only
(keep quoting the whole string when that remainder needs it, the current behaviour), and leave every
other `sheet_text` on the existing path.

## Acceptance criteria
- [ ] `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!F128` disappears from eval/baseline/poi.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests: `ADDRESS(2,3,3,TRUE,"[Book1]Sheet1")` is `[Book1]Sheet1!$C2`;
      `ADDRESS(1,1,1,TRUE,"Sheet 1")` stays `'Sheet 1'!$A$1`; `ADDRESS(1,1,1,TRUE,"Sheet2")` stays
      `Sheet2!$A$1`.
- [ ] The references.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A `[Book]` prefix with a remainder that needs quoting (`"[Book1]My Sheet"`) or a bracket elsewhere
  in the name: unprobed, keep today's output and note it as open in the references.md entry.
- Changing `quote_name` or formula stringification.

## Outcome
`address_sheet_text` in `address_areas.rs` leaves a `[book]` + plain-sheet `sheet_text` unquoted; all
other names use `quote_name` as before. poi baseline: EverythingTests!F128 fixed (no new failures).
The baseline refresh also dropped 6 unrelated poi lines (simple-monthly-budget, simple-table-named-range)
that already pass on main.
