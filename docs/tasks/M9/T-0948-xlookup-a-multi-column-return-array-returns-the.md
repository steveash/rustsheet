---
id: T-0948
title: "XLOOKUP: a multi-column return_array returns the whole matched row (2 poi known failures)"
milestone: M9
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`XLOOKUP(value, lookup_column, return_range)` where `return_range` has as many rows as the lookup
column but several columns returns the matched row of `return_range` as a 1xN array (spilling or
filling an array-formula range), as Excel does. Today XLOOKUP requires both ranges to have the same
shape and returns `#VALUE!`.

## Context
Evidence: poi/xlookup.xlsx (Excel-cached, Excel Online 16), Sheet1 (`make eval-show CASE=poi/xlookup.xlsx`).
B5:B14 = EmpIDs, C5:D14 = name/department, B2 = 8389 (row 7: "Dianne Pugh", "Finance").
- C2 `=XLOOKUP(B2,B5:B14,C5:D14)` (array formula over C2:D2): expected "Dianne Pugh"; actual `#VALUE!`.
- D2 (second cell of the same result): expected "Finance"; actual empty.
Passing today and must stay so: Sheet2 F2/F12 and Sheet3 F4 of the same workbook, and the
ironcalc-calc XLOOKUP cases.

Rule recorded in docs/excel-semantics/lookup.md "XLOOKUP: return_array may be wider than the lookup
vector".

Code: `fn_xlookup` in `crates/rustsheet-engine/src/functions/xlookup.rs`, the "Arrays must be of the
same size" check in the `CalcResult::Range` arm. For a column lookup vector, accept a return range
with the same number of rows and any number of columns and return that row as an array (a single
column keeps today's scalar result).

## Acceptance criteria
- [ ] These 2 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/xlookup.xlsx` Sheet1!C2, Sheet1!D2.
- [ ] Unit tests: a 2-column return range gives a 1x2 array; a return range with a different row
      count is still `#VALUE!`; a single-column return range still gives a scalar.
- [ ] lookup.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The horizontal analog (a row lookup vector with a multi-row return range) and array (non-range)
  return arguments: no evidence yet; file a probe follow-up rather than implementing them blind.
