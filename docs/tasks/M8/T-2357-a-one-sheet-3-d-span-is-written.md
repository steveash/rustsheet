---
id: T-2357
title: "A one-sheet 3-D span is written as the plain reference (FORMULATEXT of =ABS(Data1:Data1!A1:A3) is =ABS(Data1!A1:A3))"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT (and the displayed formula) of a formula with a one-sheet 3-D span shows the plain
sheet reference, as Excel does: `=ABS(Data1:Data1!A1:A3)` reads back as `=ABS(Data1!A1:A3)`.

## Context
probes/3d-ref-one-sheet-span-lift Calc!B6 (excel-graph, 2026-10-22, T-2351): Excel's
`FORMULATEXT(A1)` with A1 `=ABS(Data1:Data1!A1:A3)` is `=ABS(Data1!A1:A3)`; ours keeps
`Data1:Data1!`. The span already evaluates as the plain reference (`get_sheet_span_value` /
`get_sheet_span_reference` in crates/rustsheet-engine/src/reference_operators.rs) and static
analysis gives it the plain range's shape (T-2351). The rule is in docs/excel-semantics/references.md
("A scalar function over a one-sheet span's range"). Decide where to collapse it (the parser
lowering a one-sheet `SheetSpanKind` to its reference, or stringify) without changing what the
multi-sheet spans or `=@Data1:Data1!A1` (probes/3d-ref-span-edges B2) give; check that the xlsx
export of such a formula still opens.

## Acceptance criteria
- [ ] probes/3d-ref-one-sheet-span-lift Calc!B6 passes; the case leaves the baseline.
- [ ] A legacy-import probe confirms the automatic `@` T-2351 puts on a one-sheet span range
  (`SIN(Sheet1:Sheet1!A1:A10)` stored without the dynamic-array flag), or the rule is corrected.
- [ ] A unit test pins FORMULATEXT of a one-sheet span over a cell and over a range.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
