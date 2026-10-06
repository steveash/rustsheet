---
id: T-0933
title: "TEXT: an empty format string returns empty text (194 known failures)"
milestone: M10
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(value, "")` returns the empty string `""` like Excel, instead of `#VALUE!`.

## Context
The POI format-test workbooks fill whole blocks of rows with the shared formula `=TEXT(Cn, Bn)`
where both `Bn` (format) and `Cn` (value) are empty. Excel (excel-graph oracle, T-0019 sidecars in
`eval/suites/poi/oracle/`) returns `""`; we return `#VALUE!` because `format_number` rejects the
empty format code (`crates/rustsheet-engine/src/formatter/`, called from `fn_text` in
`crates/rustsheet-engine/src/functions/text/common.rs`).

Failing cells (all TypeMismatch, expected `""`, actual `#VALUE!`):
- `poi/ElapsedFormatTests.xlsx` Tests!A52:A215 (164 cells)
- `poi/FormatChoiceTests.xlsx` Tests!A27:A53 (27 cells)
- `poi/NumberFormatApproxTests.xlsx` Tests!A249 (1)
- `poi/GeneralFormatTests.xlsx` Tests!A69, A71 (2)

Same pattern, excel-cached source: `poi/FormatConditionTests.xlsx` Tests!A38, A51:A54 (5).

Evidence covers a blank value with a blank format only. Do not assume the result for a non-zero
number or text with `""`; if the implementation must decide, use the `excel-probe` skill and record
the rule in `docs/excel-semantics/` (new file `number-formats.md`).

## Acceptance criteria
- [ ] The 194 known-failure lines above disappear from eval/baseline/poi.txt with no new F lines.
- [ ] The 5 FormatConditionTests lines (A38, A51:A54) disappear too, or the Outcome explains why not.
- [ ] Unit test in the formatter or TEXT tests: `TEXT(<blank>, "")` = `""`.
- [ ] Rule recorded in `docs/excel-semantics/number-formats.md` citing the cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
Any other TEXT format feature (T-0934..T-0938).

## Outcome
`fn_text` returns `""` for a blank value with an empty format (scalar and array paths); non-blank values with `""` are unprobed and unchanged. Unit test added; number-formats.md has the `Implemented in:` line. Eval: poi known failures 889 -> 690 (199 fixed: the 194 listed plus the 5 FormatConditionTests cells), no regressions.
