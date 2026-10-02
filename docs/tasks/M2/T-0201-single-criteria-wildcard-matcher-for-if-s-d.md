---
id: T-0201
title: "Single criteria/wildcard matcher for *IF(S), D-functions and MATCH"
milestone: M2
lane: functions
status: in-progress
priority: P2
size: M
depends_on: []
---

## Goal
Ensure SUMIF/COUNTIF/AVERAGEIF(S), D* and wildcard MATCH/XLOOKUP share one criteria parser and matcher with Excel semantics (operators, wildcards ~ escapes, case-insensitivity, numeric-text coercion, blanks).

## Context
Search for existing criteria code first; consolidate rather than add.

## Acceptance criteria
- [ ] One module owns criteria; all consumers use it; spec cases for the grammar added to `smoke` (hand) and `probes` (oracle-pending).
- [ ] No eval regressions; duplication not increased.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
