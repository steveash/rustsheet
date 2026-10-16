---
id: T-1323
title: "GROUPBY sort_order, filter_array and argument errors (SPEC-0006 step 2)"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY's sort_order, filter_array and invalid-argument errors match the probe.

## Context
SPEC-0006 GROUPBY bullets sort_order / filter_array; anchors G!A145, G!J145, G!S145, G!AB145, G!A169, G!S217, G!AB217, G!A241, G!A217, G!J241. Sort by value columns pins the total rows and keeps blank keys last.

From T-1322 (GROUPBY core landed in functions/lookup_and_reference/groupby.rs): `Grouping::sort_by_keys` already takes a
direction per key field (blank last both ways, numbers exact); sort_order, filter_array and field_relationship are
`#N/IMPL!` today (the `given(...)` check in `Model::groupby`). Also score with this step:
- gen-m9/3ec66461400faf2f Calc!A17 (Excel `#VALUE!`, engine `#N/IMPL!`): `GROUPBY(Data!C1:D12,Data!C1:E12,LAMBDA(w,w),1,1,Data!C6:C6,(Data!D1:D12>1E15))`, an all-FALSE filter.
- probes/groupby-blank-values G!J49 and its AC49/AD49 companions (sort_order -1 with an empty-text key: "a", "", 0, Total).

## Acceptance criteria
- [ ] Those anchors pass (in their LAMBDA form until T-1325 lands) and leave the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
