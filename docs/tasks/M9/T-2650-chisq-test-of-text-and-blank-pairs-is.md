---
id: T-2650
title: "CHISQ.TEST of text-and-blank pairs is #DIV/0! in one shape (gen-m9 0a9226016e8f5f33 A333)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
CHISQ.TEST of text-and-blank-only pairs matches Excel in every shape.

## Context
gen-m9/0a9226016e8f5f33 Calc!A333 `=CHISQ.TEST(Data!A10:B12,Data!A8:C9)` is `#DIV/0!` in Excel, ours 1.0, while 8dd2312c96effb1b A343 (`CHITEST(Data!A6:B7,Data!C7:D8)`, no numeric pair) is 1. Probe blank-against-blank and text-against-text pairs first (excel-probe skill); see statistics.md "Paired functions" (T-2431). Also probe the degrees of freedom of different shapes: `CHISQ.TEST({1;2;3;4},{2,2;2,2})` against `CHISQ.TEST({1,2;3,4},{2;2;2;2})`, an all-numeric negative actual, and a negative expected cell with a positive total (confirm or correct the hand-derived `CHISQ.TEST({1;5;5},{-1,2,2})` = 0.082084999 unit test). Code: functions/statistical/chisq.rs.

## Acceptance criteria
- [ ] The probed rule is implemented. (gen-m9/0a9226016e8f5f33 A333 already passes: fixed by T-2094's no-number `#DIV/0!` rule; keep the degrees-of-freedom and blank-against-blank probes.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-
