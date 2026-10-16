---
id: T-1703
title: "GETPIVOTDATA without a pivot table: #REF! for references, propagate errors in the first two arguments"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GETPIVOTDATA matches Excel when the workbook has no pivot table.

## Context
probes/getpivotdata-evidence-floor (21 new-coverage failures in eval/baseline/probes.txt); docs/excel-semantics/web.md "GETPIVOTDATA without a pivot table". Errors in data_field/pivot_table propagate (both-error order unprobed), everything else is #REF!.

## Acceptance criteria
- [ ] `make eval-show CASE=probes/getpivotdata-evidence-floor` has no failing cell and the baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
