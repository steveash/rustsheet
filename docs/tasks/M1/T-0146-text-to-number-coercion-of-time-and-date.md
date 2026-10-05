---
id: T-0146
title: "Text-to-number coercion of time and date-time text (0+'10:11:12.5', VALUE)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text that Excel reads as a time or a date-time coerces to its serial wherever text is converted to
a number (`0+A1`, `-A1`, `VALUE`, arithmetic on array elements): today `cast_number` only knows
plain numbers, percentages, currencies and dates, so `0+"10:11"` is #VALUE! (Excel 0.4243055…).

## Context
- Evidence (excel-graph oracle, column C of each): `eval/suites/probes/cases/timevalue-fractional-seconds.case.json`,
  `timevalue-fractional-seconds2`, `timevalue-component-limits`, `timevalue-ampm-limits`. These are
  the probes' known failures (89 cells) recorded as new coverage by T-0910.
- Grammar: `docs/excel-semantics/dates.md#time-text`. Coercion returns the *unwrapped* serial
  (`0+"25:00"` = 1.0416666666666667, `0+"1/2/2020 10:11:12.5"` = 43832.42445023148), while
  TIMEVALUE keeps only the fractional part.
- Reuse `parse_clock_text` in `functions/date_and_time/time_text.rs` (T-0910); do not write a second clock
  parser (CLAUDE.md directive 3). Move it to a shared module if `cast.rs`/`formatter/format.rs`
  needs it (`parse_formatted_number` is the natural hook; it also returns a number format).
- Quirk: `0+" 10:11:12.5 "` (spaces around) is #VALUE! although TIMEVALUE trims it
  (timevalue-fractional-seconds2 C11); padded plain numbers do coerce (`" 2 "/1` = 2).
- The `--x` parser fold was fixed by T-0132, so the `--A1` column of
  probes/timevalue-fractional-seconds now depends only on this task.
- Add upstream IronCalc's Excel-authored `TEXT/MINUS_MINUS.xlsx` (`git show
  9b1ae3fd:xlsx/tests/calc_tests/TEXT/MINUS_MINUS.xlsx` in the upstream clone from
  `scripts/upstream-log.sh`) to `crates/rustsheet-xlsx/tests/calc_tests/TEXT/`: 72 of 73 cells pass
  since T-0132, and C12 `--"12:00"` = 0.5 needs this task (calc_tests workbooks must pass whole).
  Also restore the time assertions T-0132 dropped from `test/test_unary_operators.rs`
  (`--"12:00"` = 0.5, `--"6:00:00"` = 0.25, `--"6 PM"` = 0.75, `"12:00"+1` = 1.5).
- Unprobed edges (fidelity review of T-0910): a 4-digit minute or second with a leading zero
  (`"10:0100:00"`) and hour-only AM/PM forms (`"10 P"`, `"0 PM"`); probe them before relying on them.

## Acceptance criteria
- [ ] Every `C` cell of the four probes above passes (except any whose `0+` form depends on
  something out of scope, each listed with a reason in the Outcome).
- [ ] No regressions in gen-m1/gen-m2/ironcalc-calc; `make eval-baseline` committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DATEVALUE month-first parsing (T-0024).

## Outcome
`cast_number` falls back to the new `time_text::parse_time_serial` (clock text, hour-only AM/PM,
"date clock" text, unwrapped serial; text with surrounding spaces stays #VALUE!), so `0+`, `--`,
and array arithmetic read time text (VALUE does not: `fn_value` has its own path, see T-0153). Restored the time assertions in
`test_unary_operators.rs`. Eval delta: probes known failures 132 -> 47 (85 of the 89 cells fixed);
gen-m1/gen-m2/ironcalc-calc/poi unchanged, no regressions. Not done: the oddity
`"10:11.5:12"` (timevalue-component-limits B26/C26, see dates.md "Open") and the upstream
`TEXT/MINUS_MINUS.xlsx` calc_test, which needs a clone of the IronCalc repo (outside this session's
repository scope); filed as a follow-up. The unprobed edges (4-digit leading-zero components,
hour-only `"10 P"`) were not probed.
