---
id: T-0286
title: "Probe legacy array context beyond *IF(S) in SUMPRODUCT: IF/INDEX/CHOOSE nested in SUMPRODUCT, deeper nesting, LOOKUP/MMULT arguments"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
We know, with oracle evidence, where a legacy (non-dynamic) formula evaluates arguments as arrays with no implicit intersection, beyond *IF(S) placed directly inside SUMPRODUCT.

## Context
T-0278 made `is_array_context` (crates/rustsheet-engine/src/expressions/parser/static_analysis.rs) skip the automatic `@`
for the whole subtree under SUMPRODUCT. Its evidence (probes/ifs-legacy-sumproduct) only covers *IF(S) directly
inside SUMPRODUCT. Unprobed (criteria.md): IF, INDEX and CHOOSE nested in SUMPRODUCT (a legacy IF inside
SUMPRODUCT may still need CSE), deeper nesting such as `SUMPRODUCT(--(COUNTIF(..)>0))` and
`SUMPRODUCT(ABS(A1:A6))`, and other functions that may give an array context (LOOKUP's arrays, MMULT, INDEX's
array). Write a probe with `legacy_formulas` (spec2xlsx) off-row and on-row, with a control such as
`=B1:B6*1`, and fill it with excel-graph. The spec adapter evaluates legacy formulas as dynamic (T-1096), so
also add a fixtures/ xlsx copy if the engine's import path needs gating.

## Acceptance criteria
- [ ] An excel-graph probe covers the cases above; criteria.md (or arrays.md) records the result, with the
      "Unprobed" sentence from T-0278 replaced by the evidence.
- [ ] Any mismatch with `is_array_context` is filed as its own task (or fixed here if size S).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
