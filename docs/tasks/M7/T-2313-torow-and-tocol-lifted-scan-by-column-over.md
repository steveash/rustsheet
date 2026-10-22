---
id: T-2313
title: "TOROW and TOCOL: lifted scan_by_column over an error or array and the scan_by_column error before ignore (gen-m7)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2298]
---

## Goal
TOROW and TOCOL lift scan_by_column over an array (one element per entry, an error entry or an error array giving that error per element) and check an error scan_by_column before an invalid ignore, as Excel gives.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments (gen-m7)".

Evidence: `TOROW(COMBINA(..)=#NUM!,3,SCAN(1.005,Data!A5:C9,MIN))` spills 15 `#NUM!` (611ce3bbfab27cef A919; ours one `#NUM!`); `TOROW(Data!B3:E8,3,WRAPCOLS(Data!D3:D11,1))` lifts over the 9x1 scan array (7d67c559ce7536fa A619, ours `#N/IMPL!`; the engine gives `#N/IMPL!` for `TOROW(5,3,{1,2;3,4})` too); `TOCOL(Data!E4:E12,0,TEXTAFTER(..,FILTER(..)))` has `#VALUE!` elements where ours has `#N/A` (7d67c559ce7536fa A692); `TOCOL(Data!B1:B5,Data!B8,NA())` is `#N/A` (ours `#VALUE!` from the ignore; 8044dce50616cfe9 A1508). T-1836 lifted TOCOL's ignore and UNIQUE; TOROW's and TOCOL's scan_by_column are the rest. Code: functions/lookup_and_reference/tocol_torow.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 27 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 27 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/611ce3bbfab27cef.case.json A919 `=TOROW(COMBINA((Data!D4*Data!A10),1),3,SCAN(1.005,Data!A5:C9,_xleta.MIN))`: B919 Excel #NUM!, ours <empty>; cells (14): B919:C919, A920:C920, A921:C921, A922:C922, A923:C923
- gen-m7/7d67c559ce7536fa.case.json A619 `=TOROW(Data!B3:E8,3,WRAPCOLS(Data!D3:D11,1))`: A619 Excel "x~*", ours #N/IMPL!; cells (5): A619:E619
- gen-m7/7d67c559ce7536fa.case.json A692 `=TOCOL(Data!E4:E12,0,TEXTAFTER(Data!C11,",",FILTER(Data!B1:E12,(Data!D1:D12<>0.2),Data!C8:D9)))`: A695 Excel #VALUE!, ours #N/A; cells (7): A695, C695:D695, C697, B699, C700, C703
- gen-m7/8044dce50616cfe9.case.json A1508 `=TOCOL(Data!B1:B5,Data!B8,NA())`: A1508 Excel #N/A, ours #VALUE!; cells (1): A1508

## Acceptance criteria
- [x] Visible parity: the 27 gen-m7 cells listed under Context have Excel's value, type, error and text: these 27 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [x] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [x] `make check` green; reviewers approve.

## Out of scope
- A 1x1 array scan_by_column (T-2298).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
TOROW joins TOCOL's lifting-table entry (`lift_flatten` in tocol_torow.rs, the shape per function; TOROW's static-analysis
fallback arm and `args_signature_tocol` are gone). An error array is every lifted element's result (TOROW's evidence; TOCOL
by analogy, UNVERIFIED, probed by T-2365), and an error ignore or scan_by_column comes before an invalid one. The 7 A692
cells already passed on main (T-2314), so 20 of the 27 owned lines left the baseline here.
Eval delta: gen-m7 known failures 222 -> 202 (fixed 20), no regressions.
