---
id: T-0122
title: "3-D sheet-range references (Sheet1:Sheet3!A1) in aggregates (16 known failures)"
milestone: M1
lane: engine
status: todo
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
- [ ] The 16 known-failure lines above disappear from eval/baseline/poi.txt with no new F lines.
- [ ] 3-D references go through the shared aggregate-argument iterator (no per-function copies).
- [ ] Functions that do not accept 3-D references keep a defined error, unit-tested (cite evidence
      or use `excel-probe`; record the rule in `docs/excel-semantics/`).
- [ ] Bounded: a 3-D span over many sheets with whole columns does not materialise unbounded arrays.
- [ ] `make check` green; reviewers approve.

## Out of scope
INDIRECT with 3-D text. Moving or inserting sheets inside a span (edit semantics) goes in a follow-up.
