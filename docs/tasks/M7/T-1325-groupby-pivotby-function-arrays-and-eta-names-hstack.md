---
id: T-1325
title: "GROUPBY/PIVOTBY function arrays and eta names: HSTACK/VSTACK of functions, PERCENTOF (SPEC-0006 step 4)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1322, T-1273]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY and PIVOTBY accept eta-reduced function names and HSTACK/VSTACK arrays of functions, and PERCENTOF gets its relative set.

## Context
SPEC-0006 function forms; anchors G!A1 (and every eta anchor), G!S97, G!AB97, G!S121, G!AB121, P!A121. Depends on T-1273's eta call path; do not add a second one.

## Acceptance criteria
- [ ] The eta-form anchors of probes/groupby-pivotby sheet G and probes/m7-argument-modes F33-H33 pass and leave the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
