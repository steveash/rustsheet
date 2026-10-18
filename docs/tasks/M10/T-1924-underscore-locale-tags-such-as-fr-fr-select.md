---
id: T-1924
title: "Underscore locale tags such as [$-fr_FR] select a calendar"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1922, T-1923]
---

## Goal
A locale tag with an underscore (`[$-fr_FR]`, `[$-de_DE]`, `[$-_]`) resolves as Excel does: `TEXT(45000,"[$-fr_FR]mmmm")` is "December".

## Context
Excel reads these tags as something other than a BCP-47 tag, and they select calendars. `[$-fr_FR]` gives the Saka date with English Gregorian month names ("December|1944-12-24"). `[$-de_DE]`, `[$-en_US]` and `[$-ja_JP]` give the lunar date with English names ("February|2023-02-24"). `[$-x_y]` and `[$-fr_]` give plain en-US. `[$-_]` gives Russian names and `[$-a_b_c]` Arabic. The parse rule is unknown. Probe enough tags to find it (perhaps a hash, or a hex reading of the characters), then implement it. Evidence: probes/text-locale-calendars A135-A145, probes/text-locale-name-tables A596. Depends on the lunar and Saka calendars (T-1922, T-1923) for the dates.

## Acceptance criteria
- [ ] probes/text-locale-name-tables A596 and text-locale-calendars A135-A145 pass, and the rule is recorded in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
