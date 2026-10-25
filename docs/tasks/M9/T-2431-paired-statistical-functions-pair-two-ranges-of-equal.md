---
id: T-2431
title: "Paired statistical functions pair two ranges of equal count but different shape instead of #N/A (gen-m9, 18 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
CORREL, INTERCEPT, FORECAST, FORECAST.LINEAR, CHISQ.TEST and CHITEST pair two arrays with the same number of cells and
different shapes, as Excel does, instead of `#N/A`; only a different count is `#N/A`.

## Context
Gap triage of the T-2255 gen-m9 sweep. statistics.md "Paired functions pair two ranges of equal count whatever their shapes
(T-2255)" records it: `CORREL(Data!B11:E12,Data!D9:E12)` is 1, `INTERCEPT(Data!E5:E8,Data!B3:C4)` -0.15000001236262528,
`CHITEST(Data!D10:E12,Data!A9:C10)` 0. 17 cells; the pairing order (row by row in each array) is the natural reading, and
each cell's Excel value checks it. One cell, `INTERCEPT((OFFSET(..)*VLOOKUP(..)),Data!A6:A9)` (a61e547a1cab994c A382), is
`#VALUE!` in Excel: its known_y is an array holding an error; keep it if the pairing fix does not give it, and say so.
Code: `fn_get_two_matrices` (functions/math_and_trigonometry/sum.rs) and the CHISQ.TEST reader (functions/statistical/
chisq.rs): compare cell counts, not rows and columns.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 formulas.
Examples:
- gen-m9/0a9226016e8f5f33 Calc!A291 `=CHITEST(Data!D10:E12,Data!A9:C10)`: Excel 0.0, ours #N/A
- gen-m9/0a9226016e8f5f33 Calc!A333 `=CHISQ.TEST(Data!A10:B12,Data!A8:C9)`: Excel #DIV/0!, ours #N/A
- gen-m9/a61e547a1cab994c Calc!A382 `=INTERCEPT((OFFSET(Data!A6:B12,0,2,2,2)*VLOOKUP(3.14159,Data!C11:C12,1,Data!C12)),Data!A6:A9)`: Excel #VALUE!, ours #N/A
Cells:
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A291, A333
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A370
- gen-m9/530c636749a85209.case.json (1): Calc!A388
- gen-m9/7674c89c824aca43.case.json (1): Calc!A72
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A38, A263
- gen-m9/a54c738c697617c3.case.json (1): Calc!A291
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A156
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A382
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A185
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A32, A169
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A94, A276, A314
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A43

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): the same pairing in PEARSON, RSQ, SLOPE and STEYX (`PEARSON(Data!B11:D12,{1,1,2,2,2,3})` is -0.4999999999999999, ours `#N/A`).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 visibly off.
Examples:
- gen-m9/329b177b5d5165ad Calc!A104 `=RSQ(Data!A4:C7,Data!C2:D7)`: Excel 0.3991684810183792, ours #N/A
- gen-m9/334f3dc9ca937f10 Calc!A63 `=SLOPE(Data!A7:D9,Data!D3:E8)`: Excel 1.0000000246913243, ours #N/A
- gen-m9/38e4fc309fe199ea Calc!A26 `=PEARSON({3,3,3,3},Data!B8:C9)`: Excel #DIV/0!, ours #N/A
- gen-m9/7980ffb6095a340d Calc!A352 `=STEYX(Data!C8:E10,Data!C1:C9)`: Excel #DIV/0!, ours #N/A
- gen-m9/91e167cd8d7eb652 Calc!A304 `=STEYX(Data!A7:A12,Data!B8:D9)`: Excel #DIV/0!, ours #N/A
Cells:
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A177
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A104, A232
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A63
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A26
- gen-m9/5395b8a427764a3f.case.json (1): Calc!A191
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A352
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A338
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A45
- gen-m9/91e167cd8d7eb652.case.json (2): Calc!A304, A352
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A58
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A181
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A146
- gen-m9/c4635963fc926284.case.json (1): Calc!A39
- gen-m9/eccd03df42e40006.case.json (1): Calc!A295
- gen-m9/ede0323c389fc703.case.json (1): Calc!A75

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258): `BIN2HEX(1010101,SLOPE(Data!B7:D8,Data!C5:C10))` (6 cells each) is `#DIV/0!` (ours `#N/A`; gen-m9/76551d6fbafc7007 A174).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/76551d6fbafc7007 Calc!A174 `=BIN2HEX(1010101,SLOPE(Data!B7:D8,Data!C5:C10))`: Excel #DIV/0!, ours #N/A
Cells:
- gen-m9/76551d6fbafc7007.case.json (1): Calc!A174

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `FORECAST(<FALSE>,Data!C1:D3,Data!C10:E11)` (3x2 against 2x3) is `#DIV/0!` (ours `#N/A`) and `CHITEST(Data!D7:D9,Data!C10:E10)` (3x1 against 1x3) 0.1353352832366127 (ours `#NUM!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/b4173fe25109a238 Calc!A78 `=FORECAST(CSCH(1E+16),(FORECAST(Data!E3,Data!C1:D3,Data!C10:E11)/Data!B6),1)`: Excel #DIV/0!, ours #N/A; inner `FORECAST(Data!E3,Data!C1:D3,Data!C10:E11)` Excel #DIV/0!, ours #N/A
- gen-m9/c6cfda6bd10cae93 Calc!A375 `=CHITEST(Data!D7:D9,Data!C10:E10)`: Excel 0.1353352832366127, ours #NUM!
Cells:
- gen-m9/b4173fe25109a238.case.json (1): Calc!A78
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A375

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 34 known-failure lines (17 from T-2255, 17 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: `CORREL({1,2;3,4},{1;2;3;4})` and `INTERCEPT({1;2;3;4},{1,2;3,5})` give the paired values.
- [ ] statistics.md's entry gets an "Ours:" line naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Different counts (unchanged `#N/A`).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Paired functions pair by cell count in reading order (`fn_get_two_matrices`); CHISQ.TEST/CHITEST accept a negative actual
value. gen-m9 known failures 5344 -> 5342 after the rebase onto the earlier baseline (55 lines removed in total across gen-m9
and probes); `CHISQ.TEST(Data!A10:B12,Data!A8:C9)` (0a9226016e8f5f33 A333, `#DIV/0!`) is still a known failure, filed as a
follow-up T-2650 (needs a probe of blank-against-blank pairs). INTERCEPT A382 (a61e547a1cab994c) is `#VALUE!` as in Excel now.

Owned cells still in the baseline: 0a9226016e8f5f33 A333 (T-2650); 201d67f1d59436ed A177 (`PEARSON(Data!B11:D12,{1,1,2,2,2,3})`,
last-digit ULP only, visible parity met); ede0323c389fc703 A75 (a TTEST argument-validation cell triaged here by mistake,
re-homed to T-2655). CHISQ.TEST has no per-cell sign check, only a negative summed statistic is `#NUM!`
(A375 and f2bd5f380ec002bf A314 fixed).

## Handoff (resumed)
Tried: cell-count pairing in `fn_get_two_matrices` (done, evidenced, 0 regressions); CHISQ.TEST sign rule revised twice after
fidelity review (final: no per-cell sign check, `#NUM!` only for a negative summed statistic, fits A21/A314/A375); quirk entry
extended. `make check` green on the branch. The fidelity reviewer requested changes in three rounds (round 3's only finding,
naming PEARSON/RSQ/SLOPE/STEYX and the FORECAST probe in the statistics.md entry, is fixed in the last commit but not
re-reviewed). Next: run the full review panel once on the current head, mark the PR ready, merge. Remaining baseline cells:
A333 (T-2650), A177 (ULP), A75 (T-2655).
