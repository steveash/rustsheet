---
id: T-3108
title: "Probe T of long text reached through INDEX, OFFSET, INDIRECT, IF, a name and a multi-cell range; share the top-left helper"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for T of long text (over 255 units) reached through INDEX, OFFSET, INDIRECT, IF, a defined name and a multi-cell range; `t_of_argument` matches Excel on each.

## Context
T-2627 made T return "" for long text read from a reference. The reviewers noted only single-cell references were probed (docs/excel-semantics/text.md, REPT/T 255 units). `t_of_argument` (functions/text/t_function.rs) also inlines the body of `evaluate_node_top_left` (cast.rs) to learn whether the argument was a range; share one helper.

## Acceptance criteria
- [ ] Probe cases with oracle values for T(INDEX(..)), T(OFFSET(..)), T(INDIRECT(..)), T(IF(TRUE,D1)), T(name) and T(D1:D2) on long text; the Unprobed note in text.md is replaced by the results, the engine fixed if it differs, and the inlined range match removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
