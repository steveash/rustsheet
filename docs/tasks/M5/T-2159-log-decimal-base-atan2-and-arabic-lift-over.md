---
id: T-2159
title: "LOG, DECIMAL, BASE, ATAN2 and ARABIC lift over arrays and ranges (SPEC-0011)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: M
tier: opus
campaign: visible
depends_on: [T-2198]
---

## Goal
LOG, DECIMAL, BASE, ATAN2 and ARABIC lift over arrays and multi-cell ranges like Excel (SPEC-0011 driver), returning an array that spills or is `#SPILL!`.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2198 (research), which fills `probes/t2198-m5-lift-argument-kinds`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). None of them is in eval/catalog/lift-kinds.json: over an array they give `#N/IMPL!`, over a range `#VALUE!`. Excel returns an
array (`#SPILL!` where blocked; aac6230c A258 and 57046660 A153 spill to the right and show their first element). Rule:
docs/excel-semantics/arrays.md "LOG, DECIMAL, BASE, ATAN2, ARABIC, DOLLAR, FIXED and NUMBERVALUE return arrays over array
arguments" (added by this triage). Use the SPEC-0011 lift driver (`lift_call`, functions/lift/) as `fn_mround` does; probe each
function's argument kinds (`Lift` vs `LiftArrays`: does a multi-cell range given directly lift or refuse?) with the `excel-probe`
skill before choosing, and regenerate lift-kinds.json (`rustsheet-eval lift-kinds`).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 7 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/5704666097cbc5b2 Calc!A153 | `=LOG((INDEX(Data!B8:E12,2,0.1)^((-2.5)+Data!$A$5)),(1^Data!E12))` | "#VALUE!" | "#N/IMPL!" |
| gen-m5/043ee562f32eb901 Calc!A204 | `=DECIMAL(IF(Data!$A1,1.005,Data!A2),INDEX(Data!A1:D5,1e-15,4))` | "#SPILL!" | "#VALUE!" |
| gen-m5/1d22a002d4f380b3 Calc!A115 | `=DECIMAL((INDEX(Data!A8:D10,0,3)&VLOOKUP(Data!E$5,Data!B11:E11,(-0.0001),Data!$B2)),25)` | "#SPILL!" | "#N/IMPL!" |
| gen-m5/8a387c26d182e73c Calc!A42 | `=BASE(338,25,INDEX(Data!A5:D11,0,0))` | "#SPILL!" | "#VALUE!" |
| gen-m5/aac6230c2fb40e1e Calc!A258 | `=BASE((INDEX(Data!C10:E10,0)*(7*Data!B11)),34)` | "0" | "#N/IMPL!" |
| gen-m5/c0651f0e0fb544d0 Calc!A84 | `=ATAN2(IFS(TRUE,(Data!C12*Data!A4)),INDEX(Data!A2:A5,MIN(Data!A1:A9,Data!A7:A10),COUNTIF(Data!A9:C...` | "#SPILL!" | "#VALUE!" |
| gen-m5/f0f205865a3a4328 Calc!A208 | `=ERROR.TYPE(ARABIC(INDEX(Data!C5:E12,Data!C10,3)))` | "#SPILL!" | 3.0 |

gen-m6 triage (T-2180, excel-graph oracle, 2026-10-21): Three more cells where the callers lift over a LOG, DECIMAL or ARABIC result over a range: `COLUMNS(LOG(OFFSET(Data!A6:C6,0,2)))` is 3 (ours `#VALUE!`); `XLOOKUP(..,DECIMAL(OFFSET(Data!B10:E11,5,3,3,2),21),0)` is `#SPILL!` (DECIMAL lifts, so XLOOKUP's match_mode is an array); `TRIMRANGE(..,ARABIC(OFFSET(Data!B6:E12,1E-15,1,-0.5)),3)` is `#SPILL!` (ARABIC lifts over the OFFSET range).

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 3 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/535caabe1e635e2d Calc!A140 | `=TRIMRANGE(Data!A7:D12,ARABIC(OFFSET(Data!B6:E12,Data!E$2,1,(-0.5))),3)` | "#SPILL!" | "#VALUE!" |
| gen-m6/60bcd25271767317 Calc!A380 | `=COLUMNS(LOG(OFFSET(Data!A6:C6,0,Data!B7)))` | 3.0 | "#VALUE!" |
| gen-m6/fe0972380d030e4a Calc!A224 | `=XLOOKUP(VALUE((Data!$E$1&Data!D6)),Data!A5:A10,Data!C1:D9,ISERR(FIND(" x ","ü",1)),DECIMAL(OFFSET(Data!B10...` | "#SPILL!" | "#VALUE!" |

## Acceptance criteria
- [ ] The 3 gen-m6 known-failure lines above disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), or each remaining one is named in the Outcome with its cause.
- [x] These 7 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] arrays.md entry updated with the probed argument kinds and marked implemented for these five.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- DOLLAR, FIXED, NUMBERVALUE (T-2160).

## Outcome
- LOG, DECIMAL, BASE, ATAN2 and ARABIC run on the SPEC-0011 driver with every argument `Lift`
  (`lift/kinds/general.rs`, `lifting_table_functions!`, their static-analysis fallbacks removed;
  eval/catalog/lift-kinds.json regenerated). Each kernel reads its arguments left to right and keeps
  its old checks in order (`excel_log`, `base_text` in mathematical.rs). As in every lifted kernel,
  element errors carry no message. DECIMAL, BASE and ARABIC have their caps in
  `lift/tests/kernel_work.rs`.
- Tests: `test/test_fn_log_base_lift.rs` (array and range lifting, broadcast, `#SPILL!`, the
  scalar rules kept).
- Eval: gen-m5 known failures 107 -> 100 (the 7 cells above), probes 1738 -> 1690
  (probes/t2198-m5-lift-argument-kinds, 48 cells); regressions 0; visibly-off cells 1750 -> 1730.
- ARABIC of a blank cell stays `#VALUE!` (Excel: 0), T-2153's.
