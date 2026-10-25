---
id: T-2369
title: "Probe WORKDAY/NETWORKDAYS holidays: literal and formula number text, date text, and count-vs-skip of number-text cells"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's treatment of text holidays in WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL is known beyond a cell holding number text far from the date range, and ours matches.

## Context
T-2270 accepts a blank one-cell holidays reference and number-text cells (docs/excel-semantics/dates.md "WORKDAY and NETWORKDAYS holidays: blank and number-text cells"). Unverified there: (1) count vs skip of a number-text holiday inside the date range; (2) a literal (`"45001"`) or formula (`TEXT(45001,"0")`) number text as holidays; (3) date, currency, percent and time text in a holiday cell. `collect_serial_numbers` (functions/date_and_time.rs) currently counts all text `cast_number` reads. Use the excel-probe skill.

## Acceptance criteria
- [ ] A probes case covers (1)-(3) with oracle values; the engine matches or the findings are recorded in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
