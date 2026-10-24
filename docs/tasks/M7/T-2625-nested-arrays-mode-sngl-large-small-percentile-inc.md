---
id: T-2625
title: "Nested arrays: MODE.SNGL, LARGE, SMALL, PERCENTILE.INC and SUMPRODUCT over a GROUPBY/PIVOTBY result, and COUNT over a wide PIVOTBY (SPEC-0018 rule 4)"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2594]
---

## Goal
MODE.SNGL, LARGE, SMALL, PERCENTILE.INC and SUMPRODUCT read a nested GROUPBY/PIVOTBY cell as Excel does, and COUNT over a
PIVOTBY with several value columns counts what Excel counts.

## Context
Split from T-2594, which made SUM, COUNT and COUNTA flatten nested cells and the walker-based aggregates (`walk_aggregate_args`,
`NestedReading` in functions/util/aggregate_args.rs) `#N/A` (docs/excel-semantics/arrays.md "Aggregates over nested GROUPBY
and PIVOTBY cells"). These consumers read their arrays through other collectors and still give `#VALUE!`:
- probes/t2594-nested-aggregates P!A15 `MODE.SNGL`, A19 `LARGE(..,1)`, A20 `SMALL(..,1)`, A21 `PERCENTILE.INC(..,0.5)`:
  Excel `#N/A` (ERROR.TYPE 7), ours `#VALUE!`.
- P!A22 `SUMPRODUCT(GROUPBY(..))`: Excel 0 (the nested cells count as no number), ours `#VALUE!`.
- gen-m9/e85daedce5c2675e Calc!A35 `BESSELJ(COUNT(PIVOTBY(Data!C1:C12,Data!B1:D12,Data!B1:E12,LAMBDA(y,y)),Data!B3:B5,
  Data!D9:D10),0)`: Excel -0.06612330960046267, ours -0.05915406669571728 since T-2594 (-0.0737 before). A PIVOTBY with
  several row and value columns; trace which cells Excel's COUNT sees (`rustsheet-eval show`, then a probe of the bare COUNT
  with the excel-probe skill).
- Probe the consumers T-2594 left unprobed: VARA, STDEVA, VARPA and STDEVPA (`#N/A`, inferred from VAR.P and STDEV.S) and AND,
  OR, XOR, IMSUM and IMPRODUCT (`#VALUE!`, `NestedReading::Value` in aggregate_args.rs) over a nested result, and change
  their `NestedReading` if Excel differs.

## Acceptance criteria
- [ ] The six cells above (five done; gen-m9 A35 handed to T-2460: its COUNT differs only in blank keys) leave eval/baseline/probes.txt and gen-m9.txt (`make eval-baseline`), no new known-failure lines.
- [x] Each flatten charges SPEC-0018's walk budget; arrays.md records the rules.
- [x] `make check` green; reviewers approve.

## Out of scope
- SCAN over a GROUPBY result (T-2398).

## Outcome
- MODE.SNGL/MODE.MULT, LARGE, SMALL and the PERCENTILE/QUARTILE family give `#N/A` over a nested GROUPBY/PIVOTBY cell
  (`nested_array_error`, nested_array.rs). SUMPRODUCT reads such a cell as 0, and AND, OR and XOR read it as `#N/A`.
- New probe probes/t2625-nested-consumers (32 cells, excel-graph): the VARA family is `#N/A` and IMSUM/IMPRODUCT are `#VALUE!`,
  both as already implemented. COUNT over a wide PIVOTBY is off only because of blank keys (Excel shows "", ours 0).
  That is T-2460's rule, so gen-m9/e85daedce5c2675e A35 and the probe's ten blank-key cells are handed to T-2460.
- Eval delta: probes t2594-nested-aggregates A15, A19:A22 fixed (5), gen-m9 3 fixed (QUARTILE, PERCENTRANK and
  PERCENTRANK.EXC over a nested result: 46ac3ef5e61583ca A325, 2138b909d8f810ea A226, 928d08e3e1eecb5f A374). No regressions. New case: 22 of 32 cells pass, and the
  10 blank-key cells are baselined under T-2460.
- No new walk is added: the `#N/A` check only scans the array the consumer already holds, and the 0 is per cell, with no
  flattening, so no budget is charged.
