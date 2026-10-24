---
id: T-2594
title: "Nested arrays: MAP, BYROW, BYCOL, REDUCE, the aggregates and reference cells over a GROUPBY/PIVOTBY result (SPEC-0018)"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2592, T-2593, T-2602]
---

## Goal
MAP, BYROW, BYCOL, REDUCE and the aggregates read a nested GROUPBY/PIVOTBY cell as Excel does, and a one-cell reference body makes a reference cell (SPEC-0018 rules 4, 5 and 7).

## Context
docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md rules 4, 5 and 7 and Design "Consumers with their own rule".
- MAP, BYROW, BYCOL and REDUCE hand the LAMBDA each nested cell whole. A step that returns an array is `#CALC!` (A104).
- SUM, COUNT and COUNTA flatten nested cells. AVERAGE is `#N/A`, ARRAYTOTEXT `#CALC!`, and TEXTJOIN and CONCAT `#VALUE!`.
- A one-cell reference body makes a `Reference` cell: it displays `#VALUE!`, and INDEX reads it as the referenced value (A21).

Owned:
- probes/t2311-pivotby-value-columns-error P!D12. Excel's error is inferred (A104, A18) to come from its `IFERROR(x&"",..)` step, which stays an
  array, so `#CALC!` (rule 5; the oracle shows only "E"). Ours is a value.
- probes/t2592-nested-groupby-cells A1:A15, A21 and A100:A109, whichever of them T-2593 leaves failing.

- Handed over from T-2593 (2026-10-24): four gen-m9 aggregates over a LAMBDA(x,x)-style GROUPBY/PIVOTBY, decided by how the
  aggregates read nested cells (rule 4): gen-m9/06b4e8f5de409d5e Calc!A366 (MAX `#N/A`), 68d8ffb69ff739d8 Calc!A285 (HARMEAN `#N/A`),
  7c20351562a2d71d Calc!A381 (AVERAGE `#N/A`), e85daedce5c2675e Calc!A35 (COUNT 142).

T-2593 landed the `Reference` kind with its producer (a one-cell reference result holds the referenced value, which INDEX returns:
A21 passes) and INDEX's reads, which already make A106 and A107 pass; the walk budget is `NestedBudget` (nested_array.rs), charged
by `NestedArray::value`. If the aggregates grow the task past size S, split it (planner) first.

## Acceptance criteria
- [x] P!D12 and T-2592's remaining cells pass and leave eval/baseline, with no new known-failure lines.
- [x] Every flatten and every MAP/BYROW/BYCOL/REDUCE hand-over charges SPEC-0018's walk budget: `SUM(VSTACK(r,r,..))` and a REDUCE
      over a fan-out of a ~2^20-cell nested cell are `#ERROR!` in bounded time (unit tests).
- [x] arrays.md records each rule with its evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- SCAN over a GROUPBY result (T-2398).
- An operator over the whole result (`&""` keeps the shape, A13; what each cell holds is unprobed).

## Outcome
MAP and REDUCE (through `lambda_cells`) hand the LAMBDA each nested array cell as its array, charged to the walk budget, and
past the budget the whole call is `#ERROR!`. The aggregate walker gained `NestedReading`: SUM, COUNT and COUNTA flatten nested
cells (charged), the AVERAGE family, MIN, MAX, PRODUCT, SUMSQ and the A-variants give `#N/A`, AND/OR/XOR and IMSUM/IMPRODUCT
keep `#VALUE!`; a `Reference` cell stays `#VALUE!` everywhere (gen-m7/bcad709c30cfdf64 A13, gen-m9/2a18b2f5a4dc43ac A371).
ARRAYTOTEXT over a result holding a nested cell is `#CALC!`. New probe probes/t2594-nested-aggregates (32 excel-graph cells)
settles the other aggregates; arrays.md "Aggregates over nested GROUPBY and PIVOTBY cells".
Eval: probes −30 (all of t2592's remaining cells and t2311 P!D12) +5 new-case lines (t2594 A15, A19:A22), gen-m9 −3, gen-m7 −1,
0 regressions. Follow-up T-2625: MODE.SNGL, LARGE, SMALL, PERCENTILE.INC, SUMPRODUCT and gen-m9/e85daedce5c2675e A35 (COUNT
over a wide PIVOTBY).
