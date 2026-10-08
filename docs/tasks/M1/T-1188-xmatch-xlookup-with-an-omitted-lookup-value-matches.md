---
id: T-1188
title: "XMATCH/XLOOKUP with an omitted lookup_value matches a blank cell in linear modes"
milestone: M1
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An omitted lookup_value in XMATCH/XLOOKUP behaves exactly as a reference to a blank cell in every match and search mode, including exact, -1, 1, wildcard and reverse linear search.

## Context
probes/xlookup-omitted-lookup-value (excel-graph, T-1157): the omitted forms N1-N5 and N12-N15 (Om sheet) are 2 / 20 where the engine gives #N/A, "nf" or 1 / 10; the blank-cell twins N18-N34 pass. `linear_scan` (functions/lookup_and_reference/xmatch.rs) treats only an `EmptyCell` lookup value as blank, so `EmptyArg` must be treated the same. See docs/excel-semantics/lookup.md "XLOOKUP / XMATCH with an omitted lookup_value".

## Acceptance criteria
- [ ] Every cell of probes/xlookup-omitted-lookup-value passes and `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
