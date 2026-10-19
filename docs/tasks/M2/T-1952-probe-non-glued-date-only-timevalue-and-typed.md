---
id: T-1952
title: "Probe non-glued date-only TIMEVALUE and typed date+clock text"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for TIMEVALUE of non-glued date-only text (`"1/2/2020"`, `"July 1, 2026"`), a 1904-system pre-epoch cell, and typed non-glued date+clock text (`1/2/2020 10:00`, `2020-01-02 10:00`).

## Context
T-1907 generalised the date-only-gives-0 rule and the typed `m/d/yyyy h:mm` conversion beyond the probed glued-month shapes; dates.md marks them UNVERIFIED. Code: `functions/date_and_time/time_text.rs`, `formatter/parse_formatted.rs`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values covers the shapes above; dates.md UNVERIFIED notes resolved; implementation matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Two probe cases (probes/date-text-non-glued-timevalue, probes/date-system-1904-timevalue-non-glued) with excel-graph values. The T-1907 guesses held (date-only TIMEVALUE is 0, 1904 pre-epoch is `#VALUE!`, typed non-glued date+`H:MM` is a number with `m/d/yyyy h:mm`); the guess that typed clocks with seconds or AM/PM stay text was wrong, so `ends_with_typed_clock` (formatter/parse_formatted.rs) now accepts them. dates.md updated; no known failures added.
