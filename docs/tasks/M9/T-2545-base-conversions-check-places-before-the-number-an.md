---
id: T-2545
title: "Base conversions check places before the number: an invalid places is #NUM! over the number's error or text (gen-m9, 15 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
The base conversions with `places` check it before the number, as Excel does: an invalid places is `#NUM!` even when the number is an error or unreadable text.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "Base conversions"). Excel gives `#NUM!` for `DEC2HEX(<#VALUE!>,11)`, `DEC2BIN(<#DIV/0!>,11)`, `DEC2HEX(<#N/A>,0)`, `DEC2OCT(<#N/A>,<empty cell>)`, `HEX2BIN(<#N/A>,0)`, `HEX2BIN(<#CALC!>,-0.5)`, `HEX2OCT(<#N/A>,0)`, `HEX2OCT(<#VALUE!>,6.25E-5)`, `HEX2OCT(<#VALUE!>,0.5)`, `OCT2HEX(<#DIV/0!>,-0.5)`, `OCT2HEX(<#VALUE!>,42)`, `BIN2HEX(<#VALUE!>,0.01005)`, `BIN2OCT(<#VALUE!>,0)`, `BIN2OCT("ABC",-1)` and `DEC2OCT(" x ",-2.5)`; ours the number's error or `#VALUE!`. So places is read and range-checked (truncated, then 1 to 10; engineering.md "Engineering edge-pool smoke check (T-2249)") before the number. engineering.md "The second argument's error wins in the two-argument base conversions, DELTA and GESTEP" (T-1816) already has places' error first; this extends it to an invalid places value. Whether DELTA's and GESTEP's second argument also wins by value (not only by error) is unprobed: leave them.
Code: `number_and_places` in number_basis.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 15 known-failure lines in eval/baseline/gen-m9.txt, 15 formulas, 15 visibly off.
Examples:
- gen-m9/1068c44afcd56d53 Calc!A242 `=HEX2BIN(((Data!E11&Data!C3)&BYCOL(Data!B2:D8,LAMBDA(w,w))),Data!$E6)`: Excel #NUM!, ours #CALC!
- gen-m9/2b0324e2de2e7b74 Calc!A112 `=OCT2HEX((((-0.0001)/42)/VALUE(Data!B7)),Data!E$3)`: Excel #NUM!, ours #DIV/0!
- gen-m9/3cf48801617a72ad Calc!A47 `=DEC2HEX((CORREL({0,0,0},Data!D6)+(Data!$D12-Data!B5)),((2+Data!A6)*Data!A$10))`: Excel #NUM!, ours #N/A
- gen-m9/3cf48801617a72ad Calc!A122 `=HEX2OCT(FILTERXML((Data!E9&"ABC"),REPT(" 42 ",3)),PRODUCT(Data!C5:C12,Data!D2:D6))`: Excel #NUM!, ours #VALUE!
- gen-m9/3cf48801617a72ad Calc!A361 `=OCT2HEX(FORECAST(3.5,Data!B5:E7,((-0.5)+Data!B8)),Data!$A2)`: Excel #NUM!, ours #VALUE!
- gen-m9/408aa11aa923a4d9 Calc!A169 `=DEC2OCT((NA()-Data!C1),Data!B9)`: Excel #NUM!, ours #N/A
Cells:
- gen-m9/1068c44afcd56d53.case.json (1): Calc!A242
- gen-m9/2b0324e2de2e7b74.case.json (1): Calc!A112
- gen-m9/3cf48801617a72ad.case.json (3): Calc!A47, A122, A361
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A169
- gen-m9/493bb194c8e0b577.case.json (1): Calc!A54
- gen-m9/5b6cc12a94f7bf14.case.json (1): Calc!A328
- gen-m9/62c886e21a1042af.case.json (1): Calc!A244
- gen-m9/a7fda34502ed3030.case.json (1): Calc!A372
- gen-m9/e707d43a891ba9b5.case.json (2): Calc!A82, A186
- gen-m9/e882060cf4613302.case.json (2): Calc!A9, A106
- gen-m9/fb73d8163ad87fc6.case.json (1): Calc!A109

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 15 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `DEC2BIN(NA(),0)`, `DEC2BIN("x",11)` and `HEX2OCT(NA(),0.5)` are `#NUM!`; `DEC2BIN(NA(),3)` stays `#N/A`.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
- `number_and_places` (number_basis.rs) now range-checks places (truncated, then 1 to 10) right after reading it, before
  the number, so an invalid places is `#NUM!` over the number's error or unreadable text. The nine identical per-function
  copies of the check were deleted, leaving one definition.
- Eval: all 15 owned gen-m9 cells pass. `make eval-baseline` removed 14 lines; the 15th, e707d43a891ba9b5 Calc!A82
  (`BIN2OCT` of text), was already fixed by T-2544 (#1756). No regressions. Unit test
  `an_invalid_places_is_num_before_the_number_is_read`, with cells from the task's Excel list only. engineering.md: the
  "Base conversions" bullet and the T-1816 entry are updated.
- `fn_dec2bin_misc` (an inherited IronCalc test) pinned `DEC2BIN(TRUE,-1)`, `DEC2OCT(TRUE,-1)` and `DEC2HEX(TRUE,-1)` as
  ours `#VALUE!`, with the note "Excel here return #NUM! instead". With places checked first, ours now gives Excel's
  `#NUM!`, so the three assertions were updated to it. probes/t2545-base-places-boolean (excel-graph, 7 cells, all pass)
  confirms it, and confirms a valid places keeps the boolean's `#VALUE!`.
- `make check` found the duplication ratchet up after the nine checks were removed: the number-reading code before and
  after each check was now identical across functions. `ten_digit_value` (the digits, `#NUM!` past ten, parsed in a
  radix) replaces five copies, and quality/duplication.json drops 3028 -> 2951 (`--update`). DELTA and GESTEP stay unprobed, as the task said.
