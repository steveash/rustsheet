---
id: T-1433
title: "Probe legacy CELL(row/col/address/type, A1:A5) with the formula row inside the range (follow-up of T-1230)"
milestone: M5
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (non-array) `CELL(info_type, A1:A5)` for the info_types other than "contents" is settled by oracle evidence: whether the reference is implicitly intersected when the formula row is inside the range.

## Context
T-1230 gave CELL's `reference` argument a vector signature (`args_signature_cell` in expressions/parser/static_analysis.rs), so no legacy formula intersects it. probes/info-cell-contents-range D3/D8 show this only for "contents" (docs/excel-semantics/information.md, "CELL(\"contents\", ref) reads the formula's own sheet"). Also note that "contents" reads the formula's own sheet on excel-graph; check whether "type" does the same (`CELL("type",Data!A1)` against a blank and a text Calc!A1).

## Acceptance criteria
- [ ] A probe case has legacy twins (`legacy_formulas`) of `CELL("row",A1:A5)`, `CELL("col",B1:D1)`, `CELL("address",A1:A5)` and `CELL("type",A1:A5)`, each placed with the formula row or column inside the range, next to dynamic twins, plus cross-sheet `CELL("type", ...)`; the engine and the semantics entry match the oracle.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
