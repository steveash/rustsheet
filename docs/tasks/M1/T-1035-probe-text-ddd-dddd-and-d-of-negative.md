---
id: T-1035
title: "Probe TEXT ddd/dddd and d of negative fractional serials (-0.5, -0.0001)"
milestone: M1
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle-confirm TEXT of negative fractional serials with ddd/dddd/d.

## Context
T-0996 made `TEXT(-0.5,"dddd")` give "Saturday" (value as i64 truncates to serial 0, like `TEXT(-0.5,"d")` = "0" already); unprobed. `TEXT(-1,"dddd")` stays #VALUE!.

## Acceptance criteria
- [ ] Probe TEXT(-0.5,"dddd"), TEXT(-0.5,"d"), TEXT(-0.0001,"ddd") against the oracle; engine matches or a fix task is filed; dates.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
