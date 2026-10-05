---
id: T-0145
title: "Text-to-number coercion of time and date-time text (0+'10:11:12.5', VALUE)"
milestone: M1
lane: engine
status: todo
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
- The `--A1` column of probes/timevalue-fractional-seconds is blocked by the parser folding `--x`
  to `x` (`=--"1"` returns the text "1"); that is T-0920's `9b1ae3fd` port, not this task.

## Acceptance criteria
- [ ] Every `C` cell of the four probes above passes (except any whose `0+` form depends on
  something out of scope, each listed with a reason in the Outcome).
- [ ] No regressions in gen-m1/gen-m2/ironcalc-calc; `make eval-baseline` committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `--x` parser fold (T-0920). DATEVALUE month-first parsing (T-0024).
