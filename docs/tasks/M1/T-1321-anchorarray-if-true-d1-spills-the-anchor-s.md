---
id: T-1321
title: "ANCHORARRAY(@IF(TRUE,D1)) spills the anchor's array (engine #VALUE!; T-1285)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`ANCHORARRAY(@IF(TRUE,D1))` spills D1's array like Excel.

## Context
Excel spills 1,2,3 (D1 = SEQUENCE(3)) and prints `=@IF(TRUE,D1)#`; the engine gives #VALUE!. Failing cells probes/at-signed-operand.case.json Sheet1!H71:H73 (T-1285).

## Acceptance criteria
- [x] `probes/at-signed-operand.case.json` H71:H73 pass and leave the baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`ANCHORARRAY(@IF(TRUE,D1))` spills D1# (probes/at-signed-operand H71:H73 pass; probes baseline -3 cells, no regressions).
The `@IF` rule lives in `spill_anchor_operand` (reference_operators.rs), used only by the spill
operator, so the VLOOKUP/HLOOKUP index and LOOKUP result_vector callers of `eval_ref_or_index`
are unchanged. The `@INDEX` and `@IF` arms evaluate their operand once and intersect it with the
shared `Model::intersect_reference` (lifted from `evaluate_node_with_reference`), which removes the
2^depth re-evaluation of nested anchors; a 40-level nested test guards it. Resumed from an opus
handoff whose security and fidelity findings this fixes. Follow-up: T-1337 (probe multi-cell,
union, value and lookup-caller `@IF` cases).
