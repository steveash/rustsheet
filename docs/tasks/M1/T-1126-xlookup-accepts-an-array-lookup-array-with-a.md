---
id: T-1126
title: "XLOOKUP accepts an array lookup_array with a range return_array"
milestone: M1
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=XLOOKUP(2,{1;2;3},B1:B3)` returns B2 (a reference, so `ISREF` of it is TRUE), as in Excel.

## Context
`Model::xlookup_impl` (crates/rustsheet-engine/src/functions/xlookup.rs) only accepts a range as
`lookup_array` and gives `#N/A` ("Range expected") for an array. Excel looks the value up in the array and
picks from the return range: probes/isref-reference-lookups H10 (`=ISREF(XLOOKUP(2,{1;2;3},B1:B3))` is TRUE,
excel-graph; found in T-0165). Probe the value of the plain call and an array return_array alongside if no
case covers them yet.

## Acceptance criteria
- [ ] probes/isref-reference-lookups H10 passes; baseline shrinks.
- [ ] XLOOKUP with an array lookup_array gives Excel's values in a new or existing probe case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Array lookup_array for binary search modes beyond what the probe covers.
