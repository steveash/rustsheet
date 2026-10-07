---
id: T-0122
title: "3-D sheet-range references (Sheet1:Sheet3!A1) in aggregates (16 known failures)"
milestone: M1
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
References of the form `First:Last!A1` and `First:Last!A1:B2` cover the same cells on every
sheet from First to Last in tab order. SUM, AVERAGE, MIN, MAX, COUNT and COUNTA over them match Excel.

## Context
Today these evaluate to `#VALUE!` (or the wrong count for COUNT/COUNTA). Failing cells:
- `poi/FormulaSheetRange.xlsx` (excel-graph, T-0019) test!D11 `=SUM(Sheet2:Sheet5!A11)` expected 10,
  actual #VALUE!; test!D12 `=SUM(Sheet2:Sheet5!A12:C12)` expected 60, actual #VALUE!.
- `poi/48703.xlsx` (excel-graph) Sheet1!B2 `=SUM(Sheet1:Sheet4!C1)` expected 20, actual #VALUE!.
- `poi/55906-MultiSheetRefs.xlsx` (excel-cached) Sheet1!A3 SUM 66, B3 AVERAGE 22, C3/E3 COUNTA 3
  (actual 1), D3 COUNTA 0 (actual 1), H3 `SUM(Sheet1:Sheet3!A1:B2)` 110, I3 AVERAGE 27.5,
  B4/I4 MIN 11, B5 MAX 33, I5 MAX 44, B6 COUNT 3 (actual 0), I6 COUNT 4 (actual 0). 13 cells.
SUM has the top frequency prior (72) in eval/catalog/functions.yaml. Note `55906` includes the
formula's own sheet in the span (Sheet1:Sheet3 from Sheet1).
Touches the parser (reference grammar), the reference type and argument iteration (the one shared
aggregate iterator, T-0200). Write a short design note in the PR. If it needs a new `Node`
variant, check that the xlsx export round-trips the formula text.

## Acceptance criteria
- [x] The 16 known-failure lines above disappear from eval/baseline/poi.txt with no new F lines.
- [x] 3-D references go through the shared aggregate-argument iterator (no per-function copies).
- [x] Functions that do not accept 3-D references keep a defined error, unit-tested (cite evidence
      or use `excel-probe`; record the rule in `docs/excel-semantics/`).
- [x] Bounded: a 3-D span over many sheets with whole columns does not materialise unbounded arrays.
- [x] `make check` green; reviewers approve.

## Out of scope
INDIRECT with 3-D text. Moving or inserting sheets inside a span (edit semantics) goes in a follow-up.

## Outcome
Done 2026-10-06. Design: a sheet prefix containing `:` (impossible in a sheet name) is a 3-D span;
the lexer reads the unquoted form `First:Last!` when neither name needs quotes, the quoted form
`'First:Last'!` comes through the existing quoted-sheet path. The parser wraps the plain
reference in a new `Node::SheetSpanKind` (resolution in `parser/sheet_span.rs`). Evaluation is
opt-in: only `evaluate_aggregate_arg` turns it into a `Range` whose corners sit on the first and last
sheet, and the one shared walk `for_each_cell_or_error_in_range` walks it sheet by sheet, clipping
each to its used area. Everywhere else it is `#REF!` (value needed) or `#VALUE!` (one-sheet reference
needed), matching new probes/3d-ref-* (excel-graph). `Data!A1:Other!B2` stays `#VALUE!` explicitly in
`get_range`. Formula text round-trips (also through RC storage) and sheet renames update the span.
Eval delta: poi known failures 658 → 642 (16 fixed, 0 regressions); probes +3 cases (44 cells,
32 pass, 12 new-coverage failures owned by T-0196 and T-0197). Follow-ups: T-0196 (AVERAGEA, MAXA,
MINA, PRODUCT, CONCAT, LARGE, RANK), T-0197 (INDEX, AREAS, VLOOKUP, MATCH, COUNTBLANK give #VALUE!), T-0198 (edit
semantics).
