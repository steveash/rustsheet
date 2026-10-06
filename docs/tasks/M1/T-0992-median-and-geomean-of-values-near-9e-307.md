---
id: T-0992
title: "MEDIAN and GEOMEAN of values near 9E+307 do not overflow"
milestone: M1
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MEDIAN and GEOMEAN of a range holding 9E+307 twice are exactly 9E+307, as Excel does.

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md (Aggregate overflow), evidence probes/aggregate-overflow (T-0192, excel-graph 2026-10-06). Engine MEDIAN overflows summing the two middle values (A48/B48: #NUM!/inf); GEOMEAN gives 9.000000000000166E+307 from a log-sum evaluation (A47/B47, see numbers.md GEOMEAN last digits).

## Acceptance criteria
- [ ] probes/aggregate-overflow A47, A48, B47, B48 pass; baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
MEDIAN halves the middle pair when their sum overflows; GEOMEAN uses a scaled product root when the plain product is out of normal range. probes/aggregate-overflow A47, A48, B47, B48 now pass (probes known failures 233 -> 229).
