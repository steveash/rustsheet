---
id: T-0159
title: "VALUE: date text with a clock suffix (\"July 1, 2026 10:00\")"
milestone: M1
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`VALUE` of date text that ends in a clock time returns the date serial plus the time, as Excel
does: `VALUE("July 1, 2026 10:00")` = 46204.416666666664.

## Context
probes/date-text-grammar C30, C31, C53, C55, C63 are known failures (#VALUE!). DATEVALUE and
number coercion (`0+"1/2/2020 6 PM"`) already handle this text through
`time_text::parse_date_clock_serial` (T-0024); `fn_value` (functions/text/common.rs) only calls
`parse_formatted_number`. Rules: docs/excel-semantics/dates.md ("The en-US date-text grammar",
Clock suffix). Reuse the existing parser (one definition): VALUE trims surrounding spaces first,
unlike number coercion. Check whether plain clock text (`VALUE("10:00")`) should go the same
way; probe it before implementing.

## Acceptance criteria
- [ ] probes/date-text-grammar C30, C31, C53, C55, C63 pass; their baseline lines are removed.
- [ ] No new eval failures; unit test in the VALUE tests.
- [ ] `make check` green; reviewers approve.

## Out of scope
VALUE percent rounding (T-0929).

## Outcome
`fn_value` falls back to `time_text::parse_date_clock_serial` (now `pub(crate)`) on the trimmed text.
Eval: probes/date-text-grammar C30, C31, C53, C55, C63 fixed (probes known failures 61 -> 56), no regressions.
Plain clock text (`VALUE("10:00")`) is unprobed: tracked by T-0153 and T-0162.
