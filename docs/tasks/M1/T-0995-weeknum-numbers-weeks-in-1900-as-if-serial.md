---
id: T-0995
title: "WEEKNUM numbers weeks in 1900 as if serial 1 were a Sunday (WEEKNUM(2,2) is 2)"
milestone: M1
lane: functions-date
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WEEKNUM in 1900 matches Excel: weeks are counted as if serial 1 were a Sunday.

## Context
Rule VERIFIED in docs/excel-semantics/dates.md (Early-1900 edges), evidence probes/dates-early-1900-edges (T-0182, excel-graph 2026-10-06). Engine `fn_weeknum` (failing cells A1-A3, A5, A6, A8-A11) anchors week 1 on the real 1 Jan 1900 (a Monday): WEEKNUM(2,2) gives 1 (Excel 2), WEEKNUM(59,2), (60,2) and (61,2) give 9 (Excel 10). For serial <= 61 (the probed range; later 1900 serials are unverified, probe WEEKNUM(100,2) and WEEKNUM(366,2) first) compute week = floor((serial - first)/7) + 1 with first = 1 - ((0 - s) mod 7), s the start weekday index (Sunday 0); this covers return types 1-2 and 11-17 only (type 21 / ISO already matches, A4 and A21) and also gives WEEKNUM(0) = 0 and WEEKNUM(0,2) = 1.

## Acceptance criteria
- [ ] probes/dates-early-1900-edges A1, A2, A3, A5, A6, A8, A9, A10, A11 and their B cells (C5, C6 too) pass; the other WEEKNUM cells keep passing; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
