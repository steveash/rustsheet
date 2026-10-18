---
id: T-1907
title: "TIMEVALUE of date-only text is 0 not #VALUE!; typed date+time text (July1 10:00) becomes a number"
milestone: M2
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TIMEVALUE returns 0 for date-only text, and typed date+time text with a glued month (`July1 10:00`) becomes a number with format `m/d/yyyy h:mm`.

## Context
Oracle evidence: probes/date-text-glued-month-2 column C rows 6-8, 10-11, 13-15 (TIMEVALUE of date-only text = 0, engine #VALUE!) and O8:R8 (typed `July1 10:00` = 46204.41666..., engine leaves text); dates.md glued-month entry. Check whether non-glued date-only text (`"July 1, 2026"`) also gives 0 in TIMEVALUE (probe first if no evidence; excel-probe skill). Code: crates/rustsheet-engine/src/formatter/time_text.rs, parse_formatted.rs.

## Acceptance criteria
- [ ] Cells C6:C8, C10:C11 (C9 is in T-1906's range), C13:C15 and O8:R8 of probes/date-text-glued-month-2 pass; eval baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.

## Outcome
TIMEVALUE returns 0 for date-only text (`time_text.rs`); typed date+time text becomes a number formatted `m/d/yyyy h:mm` (`model.rs`). probes/date-text-glued-month-2 now passes 106/106 (C6:C15, O8:R8 fixed). Typed clocks with seconds: format unverified (noted in dates.md).
