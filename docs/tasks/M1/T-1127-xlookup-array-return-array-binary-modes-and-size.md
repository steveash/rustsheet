---
id: T-1127
title: "XLOOKUP: array return_array, binary modes and size edges over an array lookup_array"
milestone: M1
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`XLOOKUP(2,{1;2;3},{4;5;6})` is 5 and the binary search modes work with an array lookup_array, as in Excel.

## Context
T-1126 added `xlookup_array_lookup` (functions/xlookup.rs) for an array lookup_array with a range return_array in the linear modes. An array return_array answers `#VALUE!` and binary modes fall to `#N/A`; both are unprobed. Probe first (excel-probe skill): the value of `XLOOKUP(2,{1;2;3},B1:B3)`, `if_not_found`, a row array with `B1:D1`, mismatched sizes, a 2-D lookup array, binary modes. See lookup.md "Array lookup_array".

## Acceptance criteria
- [ ] A probe case with Excel values for the unprobed forms above passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
