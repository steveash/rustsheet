---
id: T-2579
title: "Probe-first: MAKEARRAY wider than the grid, a REDUCE whose LAMBDA returns a multi-cell reference, COMBIN over FILTER and AVEDEV over IFERROR(TRIMRANGE) (gen-m9, 4 cells)"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Each of the four array-value edges below is probed and matched (or filed as its own fix task with the probe's evidence).

## Context
From T-2436's probe of the untraced gen-m9 cells. arrays.md "Array-value edges in the gen-m9 sweep (T-2436)". Probe each first (excel-probe skill): an array value wider than 16384 columns (MAKEARRAY, SEQUENCE, then COLUMNS/INDEX of it); REDUCE whose LAMBDA returns a multi-cell reference, passed to a scalar function; COMBIN and COMBINA over a FILTER result (ours `#N/IMPL!`, an unimplemented lift path); AVEDEV, AVERAGE and COUNT over IFERROR of a reference with blanks and booleans.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): four array-value edges, each needing its own probe: `MAKEARRAY(4,DATE(1952,-2.5,123456789.123),..)` is a 4x51638 array (COLUMNS 51638, past the grid's 16384; `#SPILL!` as a cell; ours `#VALUE!`), so BETAINV over it spills; `IMSQRT(REDUCE(..,LAMBDA(y,z,TRIMRANGE(Data!A4:D10))))` is `#VALUE!` (a multi-cell reference from REDUCE; ours lifts); `COMBIN(18,FILTER(..))` is 1 (ours `#N/IMPL!`); `AVEDEV(..,IFERROR(TRIMRANGE(Data!D1:E9),..))` is 23255018.338958457 (ours 32142595.747999992, the IFERROR array's blanks or booleans counted differently).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 4 known-failure lines in eval/baseline/gen-m9.txt, 4 formulas.
Examples:
- gen-m9/4250f880ab640c49 Calc!A237 `=BETAINV(1+2^-52,(-1),XMATCH(MAX(Data!E4:E10),Data!A3:A6),MAKEARRAY(4,DATE(1952,Data!C6,Data!D12),LAMBDA(acc,y,(y-y))),NORM.S.INV(1.5))`: Excel #SPILL!, ours #VALUE!
- gen-m9/df083d85b436b49a Calc!A20 `=IMSQRT(REDUCE((Data!$A2+0.1),(Data!$A7-0.5),LAMBDA(y,z,TRIMRANGE(Data!A4:D10))))`: Excel #VALUE!, ours "9.68168303635097E-17+1.58113883008419i"
- gen-m9/c18dbf21f11093f3 Calc!A174 `=BINOM.INV(0,MATCH(Data!$A9,Data!E5:E5,0),COMBIN(18,FILTER(Data!B1:C12,(Data!D1:D12<42))))`: Excel #N/A, ours #N/A; inner `COMBIN(18,FILTER(Data!B1:C12,(Data!D1:D12<42)))` Excel 1.0, ours #N/IMPL!
- gen-m9/fec39efaba9284c3 Calc!A253 `=AVEDEV(Data!C1:C11,Data!A11:A12,IFERROR(TRIMRANGE(Data!D1:E9),(Data!A$3+(-1))))`: Excel 23255018.338958457, ours 32142595.747999992
Cells:
- gen-m9/4250f880ab640c49.case.json (1): Calc!A237
- gen-m9/df083d85b436b49a.case.json (1): Calc!A20
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A174
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A253

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 4 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
Probed all four edges (probes/t2579-array-value-edges, 52 cells, excel-graph, 2026-10-24) and recorded them in arrays.md
"Array-value edges in the gen-m9 sweep (T-2436)" and logic.md. Fixed here: COMBIN lifts over arrays and ranges like
COMBINA (`fn_combin`, lift/kinds/general.rs `COMBIN_FAMILY`); IFERROR and IFNA read a multi-cell range's blank cells, and
those of the range a direct TRIMRANGE keeps, as 0 (`if_error_like`, `blanks_as_zero`; a one-cell range keeps the empty
value). gen-m9 c18dbf21f11093f3 A174 and fec39efaba9284c3 A253 leave the baseline. Filed with the probe's evidence, as
the Goal allows: arrays wider/taller than the grid, T-2595 (owns gen-m9 4250f880ab640c49 A237 and t2579 B23, B24, B26);
IMSQRT over a REDUCE returning TRIMRANGE, T-2596 (owns gen-m9 df083d85b436b49a A20 and t2579 B49); wrapped IFERROR range
forms, T-2597 (probe). Eval: gen-m9 known failures -2, probes -24 fixed (t2436 sub-expressions) with the new case's 4
cells baselined for T-2595/T-2596, no regressions.
