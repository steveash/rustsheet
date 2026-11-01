---
id: T-3617
title: "Snap the near-zero root of the GROUPBY and PIVOTBY aggregation LAMBDA"
milestone: M7
lane: engine
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
GROUPBY and PIVOTBY snap the near-zero `+`/`-` root of their aggregation LAMBDA at every call, as Excel does, so the engine matches the cells below.

## Context
- Evidence: docs/excel-semantics/numbers.md, paragraph "GROUPBY and PIVOTBY aggregation LAMBDAs" (T-3112, probes/t3112-snap-groupby-eta).
- Failing cells: Calc!B3-B9, B12-B18 and B30-B36 of probes/t3112-snap-groupby-eta (group, total, SUM and every PIVOTBY cell; the total row snaps itself). Calc!B37, B38 (`MAP` with a bare `SUM`; `BYROW(..,SUM)`, B40 and B43, already matches) expect `#VALUE!` and the engine gives `#NAME?`.
- Reuse `Model::evaluate_formula_root` as T-3105 did for `call_lambda`.

## Acceptance criteria
- [ ] Every cell above passes and leaves eval/baseline/probes.txt.
- [ ] k = 8 cells (B19-B27) still keep 1.7763568394002505e-15.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `GROUPBY(..,SUM)` (Calc!B44, B45): `#NAME?` in both Excel (graph) and the engine, already passing.
- The helpers MAP, MAKEARRAY, BYROW, BYCOL, REDUCE, SCAN (T-3111).
