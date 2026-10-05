---
id: T-0935
title: "TEXT on text and booleans: apply the format's text section (28 known failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(text_or_boolean, fmt)` formats the value with the format's text section like Excel, instead
of returning the argument unchanged (text) or as a boolean.

## Context
`fn_text` (`crates/rustsheet-engine/src/functions/text/common.rs`) returns `CalcResult::String(s)` and
`CalcResult::Boolean(b)` as-is, also for array elements. Excel turns booleans into "TRUE"/"FALSE"
and applies the text section (the 4th section; `@` is the value) to text and booleans.

Evidence (excel-graph, T-0019):
- `poi/TextFormatTests.xlsx` formula `=TEXT(Cn, CONCATENATE(";;;", Bn))`, so `Bn` is the text section.
  On "jello": A3 `"hi"` expected "hi" (actual "jello"), A4 `"h""I"` expected "hI", A5 `\h\i`
  expected "hi", A6 `\@@\@` expected "@jello@", A7 `-@-` expected "-jello-", A10 `-@-@-` expected
  "-jello-jello-". On TRUE (A12:A20) and FALSE (A21:A29) the same formats give "TRUE", "hi",
  "-TRUE-", "-FALSE-FALSE-" and so on; actual is the boolean. 26 cells: A3:A10, A12:A29.
- `poi/GeneralFormatTests.xlsx` Tests!A72 `TEXT(TRUE,"General")` expected "TRUE", A73 expected "FALSE"
  (actual boolean). A70 `TEXT("hello","General")` = "hello" already passes.

A text section with no `@` (A3, A4, A5) prints only its literals.

## Acceptance criteria
- [ ] The 28 known-failure lines above (TextFormatTests A3:A10, A12:A29; GeneralFormatTests
      A72, A73) disappear from eval/baseline/poi.txt with no new F lines.
- [ ] Same behavior for scalar and array (spilled) arguments; unit tests for both.
- [ ] Rule recorded in `docs/excel-semantics/number-formats.md` citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
Text values with a format that has no text section, such as `TEXT("jello","0.00")`. There is no
evidence here; use `excel-probe` and file a follow-up if needed.
