---
id: T-2414
title: "Probe four unexplained gen-m7 results: VSTACK of CELL(contents), a TEXTAFTER #SPILL!, VSTACK of a lifted COMBINA, TEXTSPLIT of TRANSPOSE(value)"
milestone: M7
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
Excel evidence explains four gen-m7 results the T-2254 triage could not trace to a rule, and each of their 31 known-failure lines
gets a fix task (or is fixed here when the fix is one line).

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22). Use the excel-probe skill; put the probe's own copies of the inputs
on a Data sheet (the gen cases' Data values are quoted below).
1. `VSTACK(CELL("contents",Data!$C11),Data!B7:E12)` on Calc is one `#VALUE!` (8c4570978f18f4c2 A782, nothing spilled; 28 cells). CELL
   "contents" reads the formula's own sheet (information.md "CELL("contents", ref) reads the formula's own sheet"), and Calc!C11 is blank
   in Excel too (no sidecar entry), so the CELL should be 0 and the VSTACK a 7x4 spill, which is ours. Probe the CELL alone (bare and
   `&"|"`), `VSTACK(CELL(..),1)`, `VSTACK(0,Data!B7:E12)`, `ROWS(VSTACK(CELL(..),Data!B7:E12))`, with Calc!C11 blank and with a number
   there, and a same-sheet `CELL("contents",C11)`; re-fill once (T-2383 met a one-off excel-graph CELL answer).
2. `TEXTAFTER(CHOOSE(FLOOR.PRECISE(Data!C4,0.1),..),"",1,-INT(2^53),0,OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2))` is `#SPILL!`
   (4dac39f89709e2fc A2870; C4 blank so the CHOOSE is `#VALUE!`, A3 = 1E-15, A4 = 0, A5 = 123456789.123, A2869 and A2871 occupied); ours
   one `#VALUE!`. So Excel's result has more than one cell: the if_not_found (a lifted OFFSET over the one-cell reference DROP gives)
   or the invalid match_mode makes an array. Probe ROWS, COLUMNS and `INDEX(..,1,1)` of the TEXTAFTER, of the OFFSET alone, and of
   TEXTAFTER with match_mode -2^53 beside an array if_not_found `{1;2}`.
3. `VSTACK(COMBINA(WRAPROWS(Data!E6:E12,1,Data!$B1),4),CHOOSE(3,VALUE(Data!B3),DAYS(41658,47244)))` is one `#VALUE!` (f19581ac44da1653
   A3357, nothing spilled; E6:E12 = blank, 7, 1E-15, 2.5, "abc", blank, "?b"; B1 = 1). Ours `#N/IMPL!` (COMBINA does not lift). A lifting
   COMBINA would give a 7x1 array and VSTACK an 8x1 spill, so Excel's COMBINA over an array is one `#VALUE!`, or the CHOOSE's
   `#VALUE!` (index 3 of 2 values) collapses the stack. Probe ROWS of `COMBINA({7;1},4)`, `COMBINA(Data!E7:E8,4)`, `COMBINA(4,{1;2})` and
   `VSTACK({1;2},CHOOSE(3,1,2))`.
4. `TEXTSPLIT(TRANSPOSE(SUM(Data!B6,Data!A9:A9)),Data!A3:C7)` is the 1x2 "", "" (f7e94a817e5385ac A2882:B2882; the SUM is 0 and a
   delimiter in A3:C7 is "0"). Ours "" in one cell: TRANSPOSE of a single value is a 1x1 array in ours, and a 1x1 array text lifts
   TEXTSPLIT to one element (text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift": `COLUMNS(TEXTSPLIT({"a,b"},","))` is 1). So in
   Excel TRANSPOSE of a single value is likely that value, not an array. Probe ROWS/COLUMNS of `TEXTSPLIT(TRANSPOSE("a,b"),",")`,
   `TEXTSPLIT(TRANSPOSE(A1),",")` (A1 = "a,b") and `ISREF(TRANSPOSE(A1))`.
The quirk entries carry these as open or conflicting lines: information.md (item 1), text.md (items 2 and 4), numbers.md (item 3).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 31 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas.
- gen-m7/4dac39f89709e2fc.case.json A2870 `=TEXTAFTER(CHOOSE(FLOOR.PRECISE(Data!C4,0.1),(Data!A10*Data!E7)),"",1,-INT(2^53),0,OFFSET(Data!B9:C12,5,1,DROP(Data!A3:A5,-2,0),2))`: A2870 Excel #SPILL!, ours #VALUE!; cells (1): A2870
- gen-m7/8c4570978f18f4c2.case.json A782 `=VSTACK(CELL("contents",Data!$C11),Data!B7:E12)`: A782 Excel #VALUE!, ours 0.0; cells (28): A782:D782, A783:D783, A784:D784, A785:D785, A786:D786, A787:D787, A788:D788
- gen-m7/f19581ac44da1653.case.json A3357 `=VSTACK(COMBINA(WRAPROWS(Data!E6:E12,1,Data!$B1),4),CHOOSE(3,VALUE(Data!B3),DAYS(41658,47244)))`: A3357 Excel #VALUE!, ours #N/IMPL!; cells (1): A3357
- gen-m7/f7e94a817e5385ac.case.json A2882 `=TEXTSPLIT(TRANSPOSE(SUM(Data!B6,Data!A9:A9)),Data!A3:C7)`: B2882 Excel "", ours <empty>; cells (1): B2882

## Acceptance criteria
- [x] A probe case under eval/suites/probes with the cells above, filled from excel-graph; information.md, text.md and numbers.md record
      what it shows (open lines resolved).
- [x] Each of the 31 known-failure lines above is owned by a fix task filed by this task (listing them, with `campaign: visible`), or
      leaves eval/baseline/gen-m7.txt here with no new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- ROMAN's lift (T-2410); TEXTSPLIT's argument-error order (T-2315).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probe `probes/t2414-gen-m7-unexplained` (excel-graph, 2026-10-23) explains all four: CELL result as a VSTACK argument is `#VALUE!` (T-2457);
COMBINA lifts over arrays, the A3357 `#VALUE!` is the CHOOSE collapsing the stack, which ours already does (T-2453); OFFSET with a one-cell DROP height is one 2x2 reference
(mechanism unverified, probe-first) so the TEXTAFTER is 2x2 and spills (T-2454); TRANSPOSE of one value is that value (T-2455). Quirks recorded in information.md,
numbers.md and text.md; the 31 known-failure lines are owned by T-2453..T-2457. The probe's cells it fails today are new probes.txt lines.
