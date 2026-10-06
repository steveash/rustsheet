---
id: T-0992
title: "MEDIAN and GEOMEAN of values near 9E+307 do not overflow"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MEDIAN(9E+307,9E+307) is 9E+307 and GEOMEAN of two 9E+307 values is exactly 9E+307, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Aggregate overflow), evidence probes/aggregate-overflow (T-0192, excel-graph 2026-10-06). Engine MEDIAN overflows summing the two middle values (A48/B48: #NUM!/inf); GEOMEAN gives 9.000000000000166E+307 from a log-sum evaluation (A47/B47, see numbers.md GEOMEAN last digits).

## Acceptance criteria
- [ ] probes/aggregate-overflow A47, A48, B47, B48 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
