---
id: T-0979
title: "WORKDAY holidays is a range argument: no implicit intersection in legacy formulas (2 poi known failures)"
milestone: M9
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (pre-dynamic-array) formula `=WORKDAY(C2,6,D2:D5)` read from xlsx uses the whole holidays
range, as Excel does, instead of implicitly intersecting it to one cell.

## Context
Evidence: `poi/59106.xlsx` (desktop Excel cached values), sheet Test. C2 = 42429 (2016-02-29), holidays
D2:D5 = 42422, 42423, 42437 (2016-03-08), 42499.
- B2 `=WORKDAY(C2,6,D2:D5)`: expected 42438, actual 42437 (NumericMismatch).
- A2 `=TEXT(B2,"dd\ mmm\ yyyy")`: expected "09 Mar 2016", actual "08 Mar 2016" (follows from B2).

Root cause (found in triage): after import, B2's formula is `=WORKDAY(C2,6,@D2:D5)`. The importer adds
`@` to legacy formulas wherever the function signature says the argument is a scalar, and the
signature table in `crates/rustsheet-engine/src/expressions/parser/static_analysis.rs` has
`Function::Workday => args_signature_scalars(arg_count, 2, 1)` and
`Function::WorkdayIntl => args_signature_scalars(arg_count, 2, 2)`, so the holidays range is
intersected with row 2 and only D2 (42422) is a holiday. The same formula typed into a cell gives
42438 (fn_workday itself is right). NETWORKDAYS and NETWORKDAYS.INTL already mark holidays as
`Signature::Vector` (`args_signature_networkdays*`); follow that pattern: WORKDAY's 3rd argument is a
vector, the others stay scalars. WORKDAY.INTL's 4th argument is very likely the same, but no Excel cell
shows it (docs/excel-semantics/dates.md, "WORKDAY holidays take the whole range"): change it only if you
add an oracle-filled probe cell for a legacy `WORKDAY.INTL(...,D2:D5)` with the excel-probe skill;
otherwise leave it and note the follow-up in the Outcome.

## Acceptance criteria
- [ ] `F	poi/59106.xlsx	Test!A2` and `Test!B2` disappear from eval/baseline/poi.txt with no new F
      lines in any suite.
- [ ] A parser test (test_add_implicit_intersection.rs or alongside) shows `WORKDAY(C2,6,D2:D5)` gets
      no `@` on the holidays range, while a range in the start_date and days positions still does.
- [ ] The dates.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auditing the signature table for other functions (file a follow-up if one turns up).
