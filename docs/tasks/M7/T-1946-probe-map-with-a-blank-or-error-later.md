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
campaign: visible
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

More gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22), four more error values as MAP's later array, all `#VALUE!` in Excel: a BASE
`#NUM!` (4dac39f89709e2fc A137, ours `#NUM!`), a MOD `#DIV/0!` (21b4eef18424ce94 A1393, ours `#DIV/0!`), `DROP(Data!B5:B12,0,3)` `#CALC!`
(a470dc9e165bdb20 A3360, ours `#CALC!`) and `SEQUENCE(1,SIN(Data!E9))` `#CALC!` (f19581ac44da1653 A2477, ours `#CALC!`). The other arrays
of these MAPs differ in size, so this is not a size check (MAP pads unequal sizes, arrays.md "MAP over arrays of different sizes"): an
error value as a later array is refused with `#VALUE!` whatever the error, like a single number. Owned (by the fix task this probe files,
with the 3 T-2225 cells): 4 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas.
- gen-m7/21b4eef18424ce94.case.json A1393 `=MAP({1,2;#N/A,""},Data!A6:A10,MOD(Data!E5,((-0.5)*Data!$E7)),LAMBDA(w,x,y,SUM(Data!A6:A12,y)))`: A1393 Excel #VALUE!, ours #DIV/0!; cells (1): A1393
- gen-m7/4dac39f89709e2fc.case.json A137 `=MAP(Data!A2:C4,BASE(874,3,((-0.5)-Data!$E7)),LAMBDA(w,x,CHOOSE(3,(123456789.123-x),(Data!E5^Data!B$8))))`: A137 Excel #VALUE!, ours #NUM!; cells (1): A137
- gen-m7/a470dc9e165bdb20.case.json A3360 `=MAP(Data!B4:E9,Data!B4:C5,DROP(Data!B5:B12,0,3),_xleta.AVERAGE)`: A3360 Excel #VALUE!, ours #CALC!; cells (1): A3360
- gen-m7/f19581ac44da1653.case.json A2477 `=MAP(Data!B7:E9,Data!B11:E12,SEQUENCE(1,SIN(Data!E9)),_xleta.MIN)`: A2477 Excel #VALUE!, ours #CALC!; cells (1): A2477

## Acceptance criteria
- [ ] The 7 gen-m7 cells listed under Context (3 from the T-2225 triage, 4 from the T-2254 triage) are owned by the functions-logic fix task this probe files, which lists them with `campaign: visible` (their 7 lines in eval/baseline/gen-m7.txt leave when it is done).
- [ ] A probe case tagged M7 with excel-graph values covers the shapes above.
- [ ] arrays.md records the results. A fix task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
