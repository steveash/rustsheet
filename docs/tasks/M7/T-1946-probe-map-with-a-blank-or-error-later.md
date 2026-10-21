---
id: T-1946
title: "Probe MAP with a blank or error later array and INDEX of a LAMBDA out of range"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/arrays.md "MAP over single values" and "An uncalled LAMBDA is one value" cover, from Excel evidence,
the shapes T-1511 left unprobed.

## Context
T-1511 (probes/lambda-value-arguments) found that MAP refuses a single number, text or boolean as its second or later
array (`#VALUE!`). The engine also refuses a blank there, for example `MAP({1},,LAMBDA(x,y,x))` or `MAP({1},Z9,..)` with an
empty Z9 (a reference, so likely allowed), but lets an error value through (`MAP({1},NA(),..)`). Neither is probed. INDEX of
a LAMBDA with an index of 2 or a negative index is `#REF!` in ours, also unprobed. The cell `=MAP(1,LAMBDA(x,y,x))`
(lambda-value-arguments B6) got no excel-graph value; read it through `ERROR.TYPE(..)`. Probe these cells and record the
results. If the engine differs, file a functions-logic fix task. Code: `by_slices_source` / `is_scalar_value`
(bycol_byrow.rs), and the LAMBDA arm of `index_select` (index.rs).

gen-m7 cells (T-2225 triage, 2026-10-21): Evidence for this probe's open question (an error value as MAP's later array): Excel is `#VALUE!` where a later array is an error value (`LEFT(NA(),2)`, `LOOKUP(..)+..` = `#N/A`, a `#N/A` SWITCH), ours lets the `#N/A` through. These cells are fixed by the functions-logic fix task this probe files (or by this task, if it fixes the code).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 3 known-failure lines in eval/baseline/gen-m7.txt, 3 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/ac67253061bba16a.case.json A890 `=MAP(Data!B1,LEFT(NA(),2),Data!B5:D8,_xleta.PRODUCT)`: A890 Excel #VALUE!, ours #N/A; cells (1): A890
- gen-m7/b24ce74f08d3f91a.case.json A237 `=MAP(Data!B7:B10,Data!C2:D2,(LOOKUP(Data!E7,Data!B4:B4,Data!E4:E7)+Data!C11),LAMBDA(x,y,w,Data!D10))`: A237 Excel #VALUE!, ours #N/A; cells (1): A237
- gen-m7/b24ce74f08d3f91a.case.json A1089 `=MAP(0.5,Data!C2:C12,SWITCH(IFNA("#N/A",Data!E6),ISBLANK(Data!$B11),MIN(Data!D4:D5,Data!C6:C10,Data!A$4),123456789.123,Data!E10,Data!E6,"?b"),_xleta.AVERAGE)`: A1089 Excel #VALUE!, ours #N/A; cells (1): A1089

## Acceptance criteria
- [ ] The 3 gen-m7 cells listed under Context (T-2225 triage) are owned by the functions-logic fix task this probe files, which lists them (their 3 lines in eval/baseline/gen-m7.txt leave when it is done).
- [ ] A probe case tagged M7 with excel-graph values covers the shapes above.
- [ ] arrays.md records the results. A fix task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
