---
id: T-1703
title: "GETPIVOTDATA without a pivot table: #REF! for references, propagate errors in the first two arguments"
milestone: M6
lane: functions-lookup
status: done
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

## Outcome
`Function::Getpivotdata` (functions/lookup_and_reference/getpivotdata.rs; language tables and `language.bin` regenerated; static-analysis signature): an error in data_field or pivot_table is the result, otherwise `#REF!` (no pivot tables are read). probes/getpivotdata-evidence-floor passes 23/23; its 21 known failures leave the baseline. `functions/mod.rs` stays at its ratchet by merging four compat-alias dispatch arms.
