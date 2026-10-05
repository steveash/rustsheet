---
id: T-0947
title: "MODE: a text argument or a blank single-cell reference argument is #VALUE! (4 poi known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MODE returns `#VALUE!` when an argument is a text value (even numeric text such as `"1"`) or a
single-cell reference to a blank cell, as Excel does. Text and blanks inside a multi-cell range stay
ignored. Today such arguments are silently skipped.

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15), sheet EverythingTests,
row 936 (`make eval-show CASE=poi/FormulaEvalTestData_Copy.xlsx`). B8 = 1, B9 = 2, C9 = 1.1,
D7 = blank, E16 = "1.1" (text).
- G936 `MODE(C9,E16,"1",B8)`: `#VALUE!`; actual `#N/A`.
- H936 `MODE(B8,"1",2,3)`: `#VALUE!`; actual `#N/A`.
- K936 `MODE("1",1,#DIV/0!,2,3)`: `#VALUE!` (the text comes before the error and wins); actual `#N/A`.
- M936 `MODE(B9,2,D7)`: `#VALUE!` although 2 appears twice: the blank single-cell reference D7 makes
  it `#VALUE!`; actual 2.
Passing today and must stay so: D936/E936 `MODE(B7:B15)`/`MODE(B8:B16)` = `#N/A`, F936 `MODE(B7:C9)` = 0,
I936 `MODE(1,1,2,3)` = 1, J936 `MODE(E7:G9)` = 1, L936 `MODE(C16:E16,-1.1)` = -1.1 (blank D16 inside a range is ignored).

The blank-reference rule looks arbitrary but is Excel's (directive 1); recorded in
docs/excel-semantics/coercion.md "MODE: text arguments and blank single-cell references are #VALUE!".
Whether `E16` (a single-cell reference to text) alone also gives `#VALUE!` is not isolated by this
evidence: G936 also has the literal `"1"`. Implement only what the cells prove: text **values**
and blank single-cell references; do not change single-cell references to text unless a cell shows it.

Code: `collect_mode_values` in `crates/rustsheet-engine/src/functions/statistical/mode_functions.rs`
(shared by MODE, MODE.SNGL and MODE.MULT; the evidence is for MODE, keep the path shared).
Its `_ => {}` arm drops `CalcResult::String` (and also swallows `CalcResult::Error`; unprobed, not
this task's); `EmptyCell | EmptyArg` have their own arm. Only a blank *single-cell reference* becomes
`#VALUE!`: `EmptyArg` (`MODE(1,1,)`) must not change (no evidence), and the String arm must fire only
for non-reference nodes.

## Acceptance criteria
- [ ] These 4 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!G936, H936, K936, M936.
- [ ] Unit tests for the four forms above and for a range holding text and blanks (still ignored).
- [ ] coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Booleans as MODE arguments (no evidence).
