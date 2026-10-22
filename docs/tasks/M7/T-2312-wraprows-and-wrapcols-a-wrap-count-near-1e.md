---
id: T-2312
title: "WRAPROWS and WRAPCOLS: a wrap_count near 1E+15 is #NUM! (gen-m7)"
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
WRAPROWS and WRAPCOLS are #NUM! for a wrap_count around 1E+15 and above, as Excel gives, instead of clamping it to the vector's length.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "WRAPROWS and WRAPCOLS with a huge wrap_count".

Evidence: `WRAPROWS(Data!E6:E10,SUM(..)=1000000000000001.5,1E+15)` (7d67c559ce7536fa A384), `WRAPCOLS(Data!D5:D7,1000000000000044.1)` (8044dce50616cfe9 A1187), `WRAPCOLS(Data!E9:E12,MAX(..)=1000000000000000.1)` (c36ff5f42a1a4d47 A877) and `WRAPROWS(<date>,ODD(2^53)*1=3.78E+17,1E-15)` (7307132bb8820f17 A1372) are `#NUM!`; ours clamps the count (arrays.md "Stacked results past the grid": "WRAPCOLS and WRAPROWS first clamp wrap_count to the vector's length") and spills. `COLUMNS(WRAPROWS(SEQUENCE(16385),16385))` is 16385 in Excel, so the limit lies between 16385 and 1E+15: probe the threshold (excel-probe: 2^20, 2^31, 2^32, 2^53, 1E+15-1) before choosing it, and record it in arrays.md. Code: functions/lookup_and_reference/wrapcols_wraprows.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 13 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 13 known-failure lines in eval/baseline/gen-m7.txt, 4 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/7307132bb8820f17.case.json A1372 `=WRAPROWS((DATE(2085,3,Data!C2)+Data!$E8),(ODD(2^53)*IF(Data!C$6,1,Data!D10)),1e-15)`: A1372 Excel #NUM!, ours 67634.0; cells (1): A1372
- gen-m7/7d67c559ce7536fa.case.json A384 `=WRAPROWS(Data!E6:E10,SUM(Data!D7:D11,Data!D10:D10,0),1000000000000000.0)`: A384 Excel #NUM!, ours -1.0; cells (5): A384:E384
- gen-m7/8044dce50616cfe9.case.json A1187 `=WRAPCOLS(Data!D5:D7,(SUM(Data!D8:D8,Data!E3)+SUM((-0.0001),Data!B8:B11)))`: A1187 Excel #NUM!, ours "abc"; cells (3): A1187, A1188, A1189
- gen-m7/c36ff5f42a1a4d47.case.json A877 `=WRAPCOLS(Data!E9:E12,MAX(SIN(Data!A11),(1000000000000000.0+Data!E3)))`: A877 Excel #NUM!, ours 123456789.123; cells (4): A877, A878, A879, A880

## Acceptance criteria
- [x] Visible parity: the 13 gen-m7 cells listed under Context have Excel's value, type, error and text: these 13 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [x] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [x] `make check` green; reviewers approve.

## Out of scope
- Lifted wrap_count and pad_with (T-1835).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
- Probe probes/t2312-wrap-count-threshold (excel-graph, 2026-10-22): a wrap_count of 2^31 or more is `#NUM!` in WRAPROWS
  and WRAPCOLS, padded or not. 2^31 - 0.5 still wraps. `ODD(2^53)` is not an error, so the A1372 cell follows the
  same rule. `get_wrap_count` (wrapcols_wraprows.rs) refuses such a count before the clamp to the vector's length.
  The rule is recorded in arrays.md "WRAPROWS and WRAPCOLS with a huge wrap_count", whose threshold was UNVERIFIED
  until now.
- Eval delta: the 13 gen-m7 lines are fixed (265 to 252 known failures), and the new probe passes. No regressions.
- Unit test: `wrap_count_of_2_pow_31_is_num` (two `#NUM!` cells and two that still wrap).
