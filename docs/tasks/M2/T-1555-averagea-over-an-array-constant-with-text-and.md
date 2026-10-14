---
id: T-1555
title: "AVERAGEA over an array constant with text and a boolean: {1,a,TRUE} is 0.5"
milestone: M2
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Dynamic `AVERAGEA({1,"a",TRUE})` gives Excel's 0.5, not 0.667.

## Context
Found by T-1551: probes/legacy-value-argument-intersection-b AI2 (excel-graph) is 0.5; we give 0.6666666666666666 (T-1526
engine choice). Work out Excel's rule for text and booleans inside an array constant under AVERAGEA (probe variants:
`{1,"a"}`, `{1,TRUE}`, `{"a",TRUE}`, `{1,"1"}`), record it in docs/excel-semantics/ (statistical) and match it.

## Acceptance criteria
- [ ] AI2 of probes/legacy-value-argument-intersection-b passes and the variant probes are recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
