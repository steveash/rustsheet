---
id: T-1324
title: "GROUPBY subtotals (total_depth 2/-2) and field_relationship (SPEC-0006 step 3)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY with total_depth 2/-2 shows subtotals and "Grand Total" as Excel does, and field_relationship 0/1 follows the probe.

## Context
SPEC-0006 GROUPBY total_depth and field_relationship bullets; anchors G!J49, G!S49, G!AB49, G!A73, G!S169, G!J169 (the last pending T-1328).

From T-1322 (groupby.rs): total_depth ±2 with more than one key field returns `#N/IMPL!` from `Model::total_depth`
(`TODO(T-1324)`; ±2 with one key is already `#VALUE!`), and any given field_relationship is rejected by the `given(...)`
check in `Model::groupby`. Groups come sorted lexicographically by key prefix (`Grouping::sort_by_keys`), so the subtotal
rows go in with one linear pass.

## Acceptance criteria
- [ ] G!J49, G!S49, G!AB49, G!A73, G!S169 pass and leave the baseline; field_relationship 1 follows T-1328's evidence or stays a recorded known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
