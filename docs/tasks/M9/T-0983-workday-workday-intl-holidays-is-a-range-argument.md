---
id: T-0983
title: "WORKDAY/WORKDAY.INTL holidays is a range argument: no implicit intersection in legacy formulas (2 poi known failures)"
milestone: M9
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
In a legacy (non-dynamic-array) formula, WORKDAY's `holidays` argument (and WORKDAY.INTL's) uses the
whole range, as NETWORKDAYS already does, instead of being cut to one cell by implicit intersection.

## Context
Found by the gap triager on 2026-10-06. `poi/59106.xlsx` (Excel-cached, sheet Test):
- B2 `=WORKDAY(C2,6,D2:D5)` with C2 = 42429 (Mon 2016-02-29) and holidays D2:D5 = 42422, 42423,
  42437, 42499: expected 42438, actual 42437. Six workdays from Feb 29 end on Tue Mar 8 = 42437, which
  is the holiday D4, so Excel moves to Wed Mar 9 = 42438.
- A2 `=TEXT(B2,"dd\ mmm\ yyyy")`: expected "09 Mar 2016", actual "08 Mar 2016" (follows from B2).

Root cause: the formula is a legacy formula, so the parser inserts implicit intersection for every
argument whose signature is `Signature::Scalar`. `static_analysis.rs` gives WORKDAY
`args_signature_scalars(arg_count, 2, 1)` and WORKDAY.INTL `args_signature_scalars(arg_count, 2, 2)`,
so `D2:D5` written in row 2 becomes D2 alone (42422, before the start date) and the holiday 42437 is
never seen. `fn_workday` / `get_holiday_set` in `crates/rustsheet-engine/src/functions/date_and_time.rs`
already read a whole range correctly. NETWORKDAYS and NETWORKDAYS.INTL show the fix: their
signatures (`args_signature_networkdays`, `args_signature_networkdays_intl` in
`crates/rustsheet-engine/src/expressions/parser/static_analysis.rs`) mark `holidays` as
`Signature::Vector`. Give WORKDAY (third argument) and WORKDAY.INTL (fourth argument) the same shape;
reuse or generalize the NETWORKDAYS helpers rather than adding near-copies (CLAUDE.md directive 3).
Semantics: docs/excel-semantics/dates.md "WORKDAY holidays take the whole range".
Run `target/release/rustsheet-eval fetch --suite poi` first to get the workbook.

## Acceptance criteria
- [ ] `F	poi/59106.xlsx	Test!A2` and `F	poi/59106.xlsx	Test!B2` disappear from
      eval/baseline/poi.txt (`make eval-baseline`), with no new F lines in any baseline
      (ironcalc-calc DATE_AND_TIME/WORKDAY_WORKDAY.INTL.xlsx stays green).
- [ ] Unit test: a legacy (non-dynamic) `=WORKDAY(C2,6,D2:D5)` in row 2 skips a holiday held in D4;
      the same for `WORKDAY.INTL(C2,6,1,D2:D5)`.
- [ ] The dates.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auditing every other function signature for range arguments marked scalar (file a task if you
  find more).
