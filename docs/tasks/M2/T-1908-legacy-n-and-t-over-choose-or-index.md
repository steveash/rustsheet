---
id: T-1908
title: "Legacy N and T over CHOOSE or INDEX of a reference take no @ on the call"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
In a legacy formula, N and T over a CHOOSE or INDEX call of a reference add no `@` on the call and read the call's reference result (`=N(CHOOSE(1,Z1:Z2))` is 7, `=T(INDEX(Y1:Y2,1))` is "a"), as Excel does.

## Context
Oracle evidence: probes/legacy-n-t-reference-args-2 (excel-graph, 2026-10-18), references.md "Legacy N and T over a reference-returning call". Ours adds `@` on the call: A10/B10 give #VALUE! (expected 7 and "a") and A20:D20, L20:M20 differ in FORMULATEXT (also `=N(INDEX(Z1:Z2,@W1:W2))`: Excel keeps the `@` only on the range in the scalar slot). Code: `reads_reference_whole`, `add_legacy_intersection` in crates/rustsheet-engine/src/expressions/parser/static_analysis/legacy.rs. Found by T-1768.

## Acceptance criteria
- [ ] Cells A10:B10, A20:D20 and L20:M20 of probes/legacy-n-t-reference-args-2 pass; eval baseline shrinks; the other cells of the case still pass.

- [ ] `make check` green; reviewers approve.

## Out of scope
- Range-in-scalar-slot shapes (already match Excel).
