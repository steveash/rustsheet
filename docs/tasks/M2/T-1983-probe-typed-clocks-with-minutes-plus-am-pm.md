---
id: T-1983
title: "Probe typed clocks with minutes plus AM/PM, glued typed seconds/AM-PM, and 1904 pre-epoch text"
milestone: M2
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for typed `H:MM AM/PM` and `H:MM:SS AM/PM` date+clock text, glued typed `July1 10 AM` and `July1 10:00:30`, typed hours past 23 and fractions; the 1904 pre-epoch question needs desktop Excel (M14).

## Context
T-1952 verified `H:MM`, `H:MM:SS` and bare-hour AM/PM; the guard `ends_with_typed_clock` in `formatter/parse_formatted.rs` keeps the rest as text (UNVERIFIED in dates.md). Use the `excel-probe` skill; remove or merge the guard once probed.

## Acceptance criteria
- [ ] A probe case with oracle values and logged formats covers the shapes; dates.md and the guard match.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
