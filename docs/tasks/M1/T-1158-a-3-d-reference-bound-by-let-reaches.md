---
id: T-1158
title: "A 3-D reference bound by LET reaches the aggregates"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1064]
---

## Goal
`LET(x,Data1:Data3!A1,SUM(x))` and `SUM(LET(x,Data1:Data3!A1,x))` are 111, as in Excel; a span bound by LET is read as a value nowhere else (`LET(x,Data1:Data3!A1,x)` keeps the bare span's error).

## Context
probes/3d-ref-choose-let C3:C4 (excel-graph, 2026-10-08, T-1064) fail: `Model::bind_argument` (functions/logical/let.rs) binds a `SheetSpanKind` through `evaluate_node_with_reference`, which gives `#VALUE!` (`get_sheet_span_reference`). Only `evaluate_reference_arg` may build a `Range` across sheets (reference_operators.rs, `get_sheet_span_range`), so binding one as a plain `Binding::Value` would leak it to value readers (INDEX, ROWS, `=x`). Likely shape: a third `Binding` variant (or `Ranges::One` kept as a reference binding) that only `Model::variable_reference` hands out, every other reader seeing what the bare span gives there. Probe `=LET(x,Data1:Data3!A1,x)`, `INDEX(LET(..),1)` and `LET(x,Data1:Data3!A1,y,x,SUM(y))` first. docs/excel-semantics/references.md "3-D references outside the aggregates".

## Acceptance criteria
- [ ] probes/3d-ref-choose-let C3:C4 pass; no other probe regresses.
- [ ] Unit test: a LET-bound span read as a value and by INDEX gives the bare span's results (probed).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Spans as LAMBDA arguments unless the same change covers them.
