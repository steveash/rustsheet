---
id: T-2359
title: "GROUPBY and PIVOTBY: lifted field_headers, sort_order and filter arguments, GROUPBY widths (gen-m7 rest of T-2311)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
GROUPBY and PIVOTBY give Excel's result on the five gen-m7 formulas below (92 known-failure cells): Excel's narrower GROUPBY width, element results where field_headers, sort_order or filter_array is a lifted array, and SCAN over a GROUPBY.

## Context
Split from T-2311 (which fixed the PIVOTBY `#VALUE!` rule: arrays.md "PIVOTBY: an error column total over several value columns is #VALUE!"). Needs a probe first (excel-probe): none of these cells say why. Code: functions/lookup_and_reference/groupby.rs, pivotby.rs, group_functions.rs. Owned gen-m7 cells (T-2225 triage, excel-graph), 92 lines in eval/baseline/gen-m7.txt:

- gen-m7/18b80d3fd292d63a.case.json A1481 `=_xlfn.PIVOTBY(Data!A1:D12,Data!A1:C12,Data!A1:A12,LAMBDA(acc,1.005),FILTER(Data!B1:E12,(Data!E1:E12<>123456789.123),Data!C4:D8),-1,Data!A8:A8,2)`: A1481 Excel #VALUE!, ours #N/IMPL!; cells (44): A1481:D1481, A1482:D1482, A1483:D1483, A1484:D1484, A1485:D1485, A1486:D1486, A1487:D1487, A1488:D1488, A1489:D1489, A1490:D1490, A1491:D1491
- gen-m7/18b80d3fd292d63a.case.json A951 `=_xlfn.GROUPBY(Data!A1:B12,Data!B1:D12,LAMBDA(w,(REGEXEXTRACT(Data!A5,"[0-9]",(-1))/-Data!D10)))`: C951 Excel <empty>, ours #VALUE!; cells (36): C951:E951, C952:E952, C953:E953, C954:E954, C955:E955, C956:E956, C957:E957, C958:E958, C959:E959, C960:E960, C961:E961, C962:E962
- gen-m7/611ce3bbfab27cef.case.json A1584 `=_xlfn.GROUPBY(Data!E1:E12,Data!B1:E12,LAMBDA(y,Data!A2),1,(HSTACK(Data!B6:D7,Data!A11:D11,Data!D$1)*YEAR(12123)),Data!A4:B6,(Data!C1:C12>0.1),(ABS(0.3)/MINUTE(0.5/86400)))`: B1584 Excel #DIV/0!, ours <empty>; cells (9): B1584:E1584, A1585:E1585
- gen-m7/b24ce74f08d3f91a.case.json A1778 `=_xlfn.GROUPBY(Data!A1:B12,Data!C1:C12,LAMBDA(y,NETWORKDAYS.INTL(16616,29409,4)),WORKDAY.INTL(21155,1,SORT(Data!A11:B11),Data!B2:C7),MAP(REGEXTEST(Data!E12,Data!$D9),Data!$A$1,LAMBDA(v,y,Data!C$8)))`: A1778 Excel #VALUE!, ours #N/IMPL!; cells (2): A1778:B1778
- gen-m7/b24ce74f08d3f91a.case.json A1850 `=SCAN(_xlfn.GROUPBY(Data!C1:D12,Data!B1:D12,LAMBDA(y,y)),Data!E3,_xleta.MIN)`: A1850 Excel #N/A, ours #VALUE!; cells (1): A1850

## Acceptance criteria
- [ ] Visible parity: the 92 gen-m7 known-failure lines above disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines elsewhere (or a cell is re-split with its probe evidence).
- [ ] docs/excel-semantics/arrays.md records each rule found, with its probe cells.
- [ ] Unit tests cover each rule on a small input.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot tables (SPEC-0015).

## Outcome
Probe: probes/groupby-lifted-args (excel-graph, 2026-10-22, 129 cells). Rule (arrays.md "GROUPBY and PIVOTBY lift over an
array field_headers or total depth"): an array `field_headers` or total depth lifts GROUPBY and PIVOTBY. Each element is the
top-left cell of the element's call, or its error in argument order. A one-element array lifts too, and `sort_order` does not.
New `Model::lift_options` (lifted_options.rs) implements it for GROUPBY (`field_headers`, `total_depth`) and PIVOTBY
(`field_headers`, `row_total_depth`, `col_total_depth`). It runs one call per element class, so an array's size does not
multiply the work. Unit tests are in test_fn_groupby.rs and test_fn_pivotby.rs.
Fixed: 55 of the 92 gen-m7 cells (A1481:D1491, B1584:E1584 with A1585:E1585, A1778:B1778).
Re-split, each with probe evidence:
- T-2397 (harness): C951:E962, 36 cells. Excel's value there is `#VALUE!`, which ours already gives. The sidecar holds blanks
  because excel-graph's undecodable rich errors inside a spill are scored as expected blanks.
- T-2398: A1850, 1 cell. SCAN over a GROUPBY is a single value.
Each option is evaluated once, and each element class runs one call with the other arguments frozen to their values, so
neither an array's size nor nesting multiplies the work (security review; a 24-deep nesting test). The class loop
(`Model::lift_by_class`) is shared with GROUPBY's existing `field_relationship` lift (quality review). PIVOTBY's
`relative_to` lift stays separate because it calls no function. A follow-up probe (A177-A225) covered range lifts, blank
elements, error order and two arrays at once.
- T-2399 (new): two array options at once broadcast to 2x2 (A209:B210), which ours does not match yet.
Eval delta: gen-m7 known failures 111 -> 56 (55 fixed, 0 regressions); probes +185 cells (4 known failures: the SCAN cells
for T-2398 and A210:B210 for T-2399).
