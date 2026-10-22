---
id: T-2311
title: "PIVOTBY and GROUPBY: #VALUE! where ours spills, result widths and lifted arguments (gen-m7)"
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
PIVOTBY and GROUPBY give Excel's result on the gen-m7 cells below: #VALUE! where ours builds (and fails to spill) a pivot, Excel's result width where GROUPBY spills, and Excel's element results where an argument is an array that lifts.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "PIVOTBY: an error column total over several value columns is #VALUE! (T-2311)" (was "PIVOTBY and GROUPBY #VALUE! results (gen-m7)").

Needs a probe: the gen-m7 cells do not say why Excel is `#VALUE!`. Five PIVOTBY calls are a single `#VALUE!` where ours is `#SPILL!` (07d27aa0add0247f A708, A992; 90de8755a2e565e6 A1, A285) or spills 12x49 cells (18b80d3fd292d63a A913, `PIVOTBY(Data!B1:B12,Data!A1:B12,Data!A1:D12,LAMBDA(v,NA()))`, 588 lines). Their common traits: multi-column col_fields or values that overlap row_fields, fields holding numbers, text and booleans, and a LAMBDA that returns an error or a number. Probe (excel-probe, research lane, or in this task) PIVOTBY with overlapping fields, an error-returning function and a two-column col_fields on a small table, record the rule in arrays.md, then fix. GROUPBY: 18b80d3fd292d63a A951 is narrower in Excel (C951:E951... empty, ours `#VALUE!`), 611ce3bbfab27cef A1584 and b24ce74f08d3f91a A1778 lift an array field_headers/sort_order/filter argument (Excel spills `#DIV/0!` and `#VALUE!`, ours `#N/IMPL!` or one cell), 18b80d3fd292d63a A1481 is a PIVOTBY with an array filter (ours `#N/IMPL!`), and b24ce74f08d3f91a A1850 (`SCAN(GROUPBY(..,LAMBDA(y,y)),E3,MIN)`) is `#N/A` in Excel. Code: functions/lookup_and_reference/pivotby.rs and groupby.rs. Split the task (lifting vs the #VALUE! rule) if it does not fit one PR.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 684 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 684 known-failure lines in eval/baseline/gen-m7.txt, 10 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/07d27aa0add0247f.case.json A708 `=_xlfn.PIVOTBY(Data!A1:B12,Data!A1:B12,Data!B1:D12,LAMBDA(z,COUNTIFS(Data!B1:E12,"?b?",Data!B11:E12,"<>")%))`: A708 Excel #VALUE!, ours #SPILL!; cells (1): A708
- gen-m7/07d27aa0add0247f.case.json A992 `=_xlfn.PIVOTBY(Data!A1:D12,Data!E1:E12,Data!D1:E12,LAMBDA(x,(WORKDAY.INTL(2137,15)/COUNT(x))))`: A992 Excel #VALUE!, ours #SPILL!; cells (1): A992
- gen-m7/18b80d3fd292d63a.case.json A913 `=_xlfn.PIVOTBY(Data!B1:B12,Data!A1:B12,Data!A1:D12,LAMBDA(v,NA()))`: A913 Excel #VALUE!, ours ""; cells (588): A913:AW913, A914:AW914, A915:AW915, A916:AW916, A917:AW917, A918:AW918, A919:AW919, A920:AW920, A921:AW921, A922:AW922, A923:AW923, A924:AW924
- gen-m7/18b80d3fd292d63a.case.json A1481 `=_xlfn.PIVOTBY(Data!A1:D12,Data!A1:C12,Data!A1:A12,LAMBDA(acc,1.005),FILTER(Data!B1:E12,(Data!E1:E12<>123456789.123),Data!C4:D8),-1,Data!A8:A8,2)`: A1481 Excel #VALUE!, ours #N/IMPL!; cells (44): A1481:D1481, A1482:D1482, A1483:D1483, A1484:D1484, A1485:D1485, A1486:D1486, A1487:D1487, A1488:D1488, A1489:D1489, A1490:D1490, A1491:D1491
- gen-m7/18b80d3fd292d63a.case.json A951 `=_xlfn.GROUPBY(Data!A1:B12,Data!B1:D12,LAMBDA(w,(REGEXEXTRACT(Data!A5,"[0-9]",(-1))/-Data!D10)))`: C951 Excel <empty>, ours #VALUE!; cells (36): C951:E951, C952:E952, C953:E953, C954:E954, C955:E955, C956:E956, C957:E957, C958:E958, C959:E959, C960:E960, C961:E961, C962:E962
- gen-m7/611ce3bbfab27cef.case.json A1584 `=_xlfn.GROUPBY(Data!E1:E12,Data!B1:E12,LAMBDA(y,Data!A2),1,(HSTACK(Data!B6:D7,Data!A11:D11,Data!D$1)*YEAR(12123)),Data!A4:B6,(Data!C1:C12>0.1),(ABS(0.3)/MINUTE(0.5/86400)))`: B1584 Excel #DIV/0!, ours <empty>; cells (9): B1584:E1584, A1585:E1585
- gen-m7/90de8755a2e565e6.case.json A1 `=_xlfn.PIVOTBY(Data!A1:D12,Data!A1:D12,Data!A1:D12,LAMBDA(z,WORKDAY.INTL(26624,-19,5,z)))`: A1 Excel #VALUE!, ours #SPILL!; cells (1): A1
- gen-m7/90de8755a2e565e6.case.json A285 `=_xlfn.PIVOTBY(Data!C1:C12,Data!C1:E12,Data!A1:B12,_xleta.AVERAGE)`: A285 Excel #VALUE!, ours #SPILL!; cells (1): A285
- gen-m7/b24ce74f08d3f91a.case.json A1778 `=_xlfn.GROUPBY(Data!A1:B12,Data!C1:C12,LAMBDA(y,NETWORKDAYS.INTL(16616,29409,4)),WORKDAY.INTL(21155,1,SORT(Data!A11:B11),Data!B2:C7),MAP(REGEXTEST(Data!E12,Data!$D9),Data!$A$1,LAMBDA(v,y,Data!C$8)))`: A1778 Excel #VALUE!, ours #N/IMPL!; cells (2): A1778:B1778
- gen-m7/b24ce74f08d3f91a.case.json A1850 `=SCAN(_xlfn.GROUPBY(Data!C1:D12,Data!B1:D12,LAMBDA(y,y)),Data!E3,_xleta.MIN)`: A1850 Excel #N/A, ours #VALUE!; cells (1): A1850

## Acceptance criteria
- [ ] Visible parity: the 684 gen-m7 cells listed under Context have Excel's value, type, error and text: these 684 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [ ] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot tables (SPEC-0015).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
Probed the cause (probes/t2311-pivotby-value-cause replays the three gen-m7 Data sheets with one argument varied at a time; probes/t2311-pivotby-value-columns-error isolates the rule on a 4-record table, excel-graph 2026-10-22): with one function over several value columns, an error in any Total column cell (hidden `""` or shown, the grand total included) makes PIVOTBY a single `#VALUE!`; group and Total row cells keep their errors. `check_totals` in pivotby.rs computes those cells for every line (`aggregate_result` gives the function's own result, so an array or reference result's `#VALUE!` does not count). arrays.md entry rewritten (VERIFIED). Eval delta: gen-m7 857 → 265 known failures (−592: the five PIVOTBY formulas, A913's 588 cells included); probes +2 cases (322 cells, 4 new-coverage failures filed as T-2360). The GROUPBY and lifted-argument cells (92 lines: A951, A1584, A1778, A1850, A1481) were split to T-2359, as the task allowed.
