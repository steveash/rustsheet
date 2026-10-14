---
id: T-1527
title: "Probe @XLOOKUP/@TAKE as ANCHORARRAY operand, LOOKUP result_vector and lookup index argument"
milestone: M1
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`@XLOOKUP(..)` and `@TAKE(..)` as the operand of `#`/ANCHORARRAY, LOOKUP's `result_vector` and a lookup index argument match Excel, backed by an oracle probe.

## Context
T-1509 made `@XLOOKUP`/`@TAKE` keep the reference for ISREF and LET only (`eval_at_picked_reference`). The other callers of `eval_ref_or_index` (spill_anchor_operand, lookup.rs result_vector, index_arg.rs) read values and have no evidence. Probe e.g. `ANCHORARRAY(@TAKE(D1:D3,1))` with D1 `=SEQUENCE(3)`, `LOOKUP(99,A1:A3,@TAKE(F1:F3,1))`, and in-row `ISREF(@TAKE(B1:B3,2))` in row 2, `ISREF(@XLOOKUP(9,A1:A3,B1:B3,5))`.

## Acceptance criteria
- [x] A probe case covers those forms; the engine matches the oracle; references.md UNVERIFIED note resolved.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/anchorarray-lookup-implicit-xlookup-take` (17 cells, excel-graph). Excel keeps the reference for `ANCHORARRAY(@TAKE(..))` and LOOKUP `result_vector`; the index-argument forms already matched. Engine now routes `@XLOOKUP`/`@TAKE` through `eval_at_picked_reference` in `spill_anchor_operand` and `lookup_result`. Documented in references.md.
