---
id: T-1158
title: "A 3-D reference bound by LET or LAMBDA reaches the aggregates"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1064]
---

## Goal
`LET(x,Data1:Data3!A1,SUM(x))` and `SUM(LET(x,Data1:Data3!A1,x))` are 111, as in Excel; a span bound by LET is read as a value nowhere else (`LET(x,Data1:Data3!A1,x)` keeps the bare span's error).

## Context
probes/3d-ref-choose-let C3:C4 and 3d-ref-let-values C7 (excel-graph, 2026-10-08, T-1064) fail. A span bound through IF or CHOOSE (and as a LAMBDA argument) is bound to `#VALUE!` by `Model::bind_argument`, which matches every value reader (3d-ref-let-values C1:C6) but not an aggregate (C7 = 111). A written-out span is bound as a value: `Model::bind_argument` (functions/logical/let.rs) binds a `SheetSpanKind` through `evaluate_node_with_reference`, which gives `#VALUE!` (`get_sheet_span_reference`). Only `evaluate_reference_arg` may build a `Range` across sheets (reference_operators.rs, `get_sheet_span_range`), so binding one as a plain `Binding::Value` would leak it to value readers (INDEX, ROWS, `=x`). Likely shape: a third `Binding` variant (or `Ranges::One` kept as a reference binding) that only `Model::variable_reference` hands out, every other reader seeing what the bare span gives there. Value readers are already probed (3d-ref-let-values C1:C6, C11: all `#VALUE!`); probe `LET(x,Data1:Data3!A1,y,x,SUM(y))` and `LAMBDA(x,SUM(x))(Data1:Data3!A1)` first. docs/excel-semantics/references.md "3-D references outside the aggregates".

## Acceptance criteria
- [x] probes/3d-ref-choose-let C3:C4 and 3d-ref-let-values C7 pass; 3d-ref-let-values C1:C6 still pass.
- [x] Unit test: a LET-bound span read as a value and by INDEX gives the bare span's results (probed).
- [x] `make check` green; reviewers approve.

## Out of scope
- Nothing beyond spans bound by LET or LAMBDA.

## Outcome
A span bound by LET or LAMBDA, written out or from IF or CHOOSE, is now a `Binding::Span` (reference_operators.rs).
`Model::variable_reference` hands it to the aggregates as the cross-sheet range, and every value read gives the bare span's
`#VALUE!`. ROWS/COLUMNS (`rows_columns_extent`) and COUNTBLANK refuse a 3-D range through the new
`RefArg::into_single_sheet_area`, as they refuse the written span. A first try that made `RefArg::into_single_area` refuse
every 3-D range regressed eight probe cells, so only these two callers refuse it. A new probe,
probes/3d-ref-let-bound-span (15 cells, excel-graph 2026-10-09), covers a second variable, a LAMBDA, COUNT, INDEX,
ROWS, ISREF and MAX next to the bare span. A review-round probe, probes/3d-ref-let-bound-span-readers (12 cells), covers
CONCAT, TEXTJOIN, AGGREGATE, AREAS, a union and COUNTBLANK; it caught COUNTBLANK counting -2, which is now fixed. Eval delta:
probes fixed 3 (3d-ref-choose-let C3:C4, 3d-ref-let-values C7), two new cases all passing, 0 regressions.
