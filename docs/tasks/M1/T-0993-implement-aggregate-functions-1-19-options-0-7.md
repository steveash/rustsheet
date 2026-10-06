---
id: T-0993
title: "Implement AGGREGATE (functions 1-19, options 0-7)"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
AGGREGATE evaluates function numbers 1-19 with the ignore options 0-7, as Excel does.

## Context
The engine returns #NAME? for AGGREGATE. First evidence: probes/aggregate-overflow A27-A29 (T-0192): AGGREGATE(9,6,F1:F3) and AGGREGATE(1,6,F1:F2) overflow to #NUM!, AGGREGATE(9,6,F1:F3,NA()) is #N/A. Probe the other function numbers, options and the array form (14-19) with the excel-probe skill before implementing; reuse the SUBTOTAL and statistics code.

## Acceptance criteria
- [ ] probes/aggregate-overflow A27-A29, B27-B29, C27-C29 pass; further probe cases cover each function number and option.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
