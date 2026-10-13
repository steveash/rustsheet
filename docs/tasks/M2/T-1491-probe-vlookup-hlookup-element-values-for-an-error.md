---
id: T-1491
title: "Probe VLOOKUP/HLOOKUP element values for an error index or lookup_value with a lifting range_lookup"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with an excel-probe what each element of a lifted VLOOKUP/HLOOKUP is when lookup_value or the index is a scalar error and range_lookup is a multi-cell range or array, with room to spill.

## Context
T-1468 made the call lift (gen-m2 Calc!A296, Calc!A381 give `#SPILL!`), but both cells collide, so the element values were never seen. docs/excel-semantics/lookup.md lists them as unprobed. The engine gives the error itself per element; Excel may give `#REF!` for an error index, as for an array index whose first element is an error. Code: `Model::lookup_head`, `LookupIndexArg::Failed`.

## Acceptance criteria
- [x] A probe such as `VLOOKUP(1,A1:A2,1/0,C1:C2)`, `VLOOKUP(1,A1:A2,"a",C1:C2)` and `VLOOKUP(1/0,A1:A2,1,C1:C2)`, and `VLOOKUP(1/0,A1:A2,1,{TRUE;TRUE})` for an array range_lookup, with free spill rows, oracle values recorded.
- [x] lookup.md and the engine match the evidence; the hand-expectation test in test_fn_vlookup.rs asserts exact values.
- [x] `make check` green; reviewers approve.

## Out of scope
- Error precedence between several failing arguments.

## Outcome
Probed `probes/vlookup-error-arg-lifting-range-lookup` (22 cells, excel-graph): with a lifting range_lookup every row is the scalar error itself (`#DIV/0!` for an error lookup_value or index, `#N/A` for `NA()`, `#VALUE!` for a text index), not `#REF!`. The engine already matched; no engine change. lookup.md updated, test_fn_vlookup.rs now asserts exact values. Eval delta: +22 probe cells passing, none failing.
