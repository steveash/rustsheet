---
id: T-1986
title: "Match the cell format of typed date text whose clock is rejected (13:00 PM gets m/d/yyyy)"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed text that stays text because its clock is rejected carries Excel's cell format, matched by the engine.

## Context
probes/date-text-typed-clock-shapes N14: `1/2/2020 13:00 PM` stays text but the cell format is `m/d/yyyy` (glued `10:00AM` stays General); see docs/research/typed-clock-shapes-2026-10-19.md and dates.md. Found by T-1979. Check whether the format is a fill artifact before implementing.

## Acceptance criteria
- [ ] Format of rejected-clock typed text matches the ENTERED log (or the finding is recorded as an artifact).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Not a fill artifact: a second probe (probes/date-text-rejected-clock-formats, 16 shapes) shows the format follows a rule. Typed text that is a date plus a number-like clock and a rejected spaced AM/PM stays text with its date part's format (`m/d/yyyy`, `d-mmm-yy` for a month name); every other rejected shape stays General. Implemented in `rejected_clock_text_format` and applied in `set_user_input` (through the new `style_with_typed_format`, shared with the number path). Rule in docs/excel-semantics/dates.md. Eval delta: one new probe case (values only, all passing); the format check is a unit test.
