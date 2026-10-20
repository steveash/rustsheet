---
id: T-2097
title: "Displayed text: the minus goes before a leading literal in a General section (\"x\"General on -1.5 is -x1.5)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A negative number formatted with `General` after a literal shows the minus in front of the literal,
as Excel does: `"x"General` on -1.5 is "-x1.5", not "x-1.5". This applies to a cell's displayed text
and to TEXT().

## Context
Gap triage 2026-10-20. Rule: docs/excel-semantics/number-formats.md, displayed-text entry: "A minus
goes before a leading literal: `"x"General` on -1.5 is "-x1.5"". That entry points to T-1614, but
T-1614 is a probe of `General` next to digit placeholders, dates and `%`, and its acceptance does not
cover these cells. Evidence: suite gen-m10 (excel-graph `Range.text`, 2026-10-16, T-1611). All 15
failing text cells under `"x"General` are this bug, and every negative value fails. Excel vs ours,
listed in the "Displayed text" section of `make eval`'s report.md:
- 039accabcfe1e6c7 F!N1 (-1.5) "-x1.5" / "x-1.5"; F!N9 (-1) "-x1" / "x-1".
- 03a284c57e24ff37 F!S2 (-123456.789) "-x123456.789"; F!S5 (-2.5); F!S10 (-1.5); F!S18 (=-1/7)
  "-x0.142857143".
- 9a8b7d68507d7ada F!I1 (-0.5) "-x0.5" / "x-0.5"; F!I4 (-45000); F!I5 (-2.5); F!I7 (-1.5).
- bfbcd16cee266336 F!D9 (=-1/7); F!D11 (-45000); F!D18 (-1); F!D19 (-1E15) "-x1E+15"; F!D20 (-2.5).
Positive values already pass, and so do the four TEXT() cells on sheet T with this code (039accab
T!A14, 9a8b7d68 T!A9, bfbcd16c T!A4, 03a284c5 T!A19; all positive). Text cells are not baselined
(decision 0068), so progress is read from report.md.
Code: crates/rustsheet-engine/src/formatter/format.rs, `ParsePart::Number` branch. When the section
has a `TextToken::General` (`has_general`), the leading minus is skipped. `TextToken::General` then
prints `general_text(general_value, ..)` with its own sign after the literals. Instead, put the minus
first and print the absolute value at the General token. Decide the sign from `general_value < 0.0`,
not from `is_negative`: `is_negative` uses the section's decimal precision, so -0.5 would lose its
minus, and Excel shows "-x0.5" (9a8b7d68 F!I1). `TEXT(5,"x General")` and `TEXT(5,"General x")`
(probes/text-format-bare-literals A75, A76) must keep passing.

## Acceptance criteria
- [ ] The 15 gen-m10 text cells above pass in the report's "Displayed text" section. No other
      gen-m10 or probes text cell that passes today fails.
- [ ] No gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] A unit test covers `TEXT(-1.5,"""x""General")` = "-x1.5" and `TEXT(-0.5,"""x""General")` =
      "-x0.5", citing the gen-m10 cells.
- [ ] number-formats.md's entry names T-2097 as the implementation of the minus rule, not T-1614.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` after a digit placeholder, a date token or `%`, and a second `General` (T-1614's probe).
  A negative value with `General` before the literal (`General x`) is unprobed: keep its current output.
