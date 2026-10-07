---
id: T-1077
title: "Probe FORMULATEXT of a union in a legacy (non-array) formula (SPEC-0004 row 16)"
milestone: M1
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what FORMULATEXT shows for a union written as a legacy (non-array) formula: does Excel add `@`?

## Context
T-1061 could not probe it: `spec2xlsx.py` writes every formula as a dynamic-array formula (probes/multiarea-at shows `=(A1,B2)` and `=SUM((A1,B2))` without `@` only for that kind). Add a spec2xlsx option (a per-cell or per-case flag) that writes a plain `<f>` with no `cm`, probe `=(A1,B2)`, `=SUM((A1,B2))` and `=@(A1,B2)` with FORMULATEXT, record the result in references.md "Multi-area references in functions", and file an engine follow-up adding `@` to the printed formula if Excel does (SPEC-0004 "add_implicit_intersection and OpUnionKind").

## Acceptance criteria
- [x] Legacy-formula probe case with oracle values kept; references.md and SPEC-0004 row 16 updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
Added a per-sheet `legacy_formulas` list to the case format (spec2xlsx writes those cells as plain
`<f>`; the engine adapter ignores it). Probe `probes/multiarea-at-legacy` (oracle: excel-graph) shows
Excel prints no `@` for a union in a legacy formula, identical to the dynamic-array result, with
positive controls (legacy `=A1:A2`, `=ABS(A1:A2)` print `@`). So the engine needs no union `@` change.
references.md and SPEC-0004 row 16 updated. Eval delta: 10 new scored cells, 5 new known failures
(H5 is the known SUM-of-union gap; H7-H10 fail because the adapter ignores `legacy_formulas`,
follow-up T-1096).
