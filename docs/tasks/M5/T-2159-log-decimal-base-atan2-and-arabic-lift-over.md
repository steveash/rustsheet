---
id: T-2159
title: "LOG, DECIMAL, BASE, ATAN2 and ARABIC lift over arrays and ranges (SPEC-0011)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: M
tier: opus
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

## Acceptance criteria
- [ ] These 7 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] arrays.md entry updated with the probed argument kinds and marked implemented for these five.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DOLLAR, FIXED, NUMBERVALUE (T-2160).
