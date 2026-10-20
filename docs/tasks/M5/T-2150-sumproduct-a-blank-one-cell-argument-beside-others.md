---
id: T-2150
title: "SUMPRODUCT: a blank one-cell argument beside others is #VALUE!, and an error argument beats a shape mismatch"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2200]
---

## Goal
SUMPRODUCT is `#VALUE!` when, among two or more arguments, a one-cell reference to a blank cell stands beside single values, and an error in any argument beats a shape mismatch between earlier ones.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2200 (research), which fills `probes/t2200-blank-and-padded-m5-args`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Rule and evidence: docs/excel-semantics/coercion.md "SUMPRODUCT: a blank one-cell argument beside other arguments is `#VALUE!`"
(added by this triage). Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/sumproduct.rs. Ours reads the blank as a 0
factor (result 0) and checks shapes argument by argument, so `SUMPRODUCT(1,B9:E10,NA())` is `#VALUE!` where Excel evaluates all
arguments and returns `#N/A` (399071ff A145). The same blank-one-cell rule holds for SUMX2MY2/SUMX2PY2/SUMXMY2 (T-0961); share
the "blank one-cell argument" test if both land. Probe before generalising: a blank one-cell argument beside a multi-cell range,
and a one-cell text/boolean argument (`excel-probe` skill), since the cells here only cover single values.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 9 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/c688ca172fe533dc Calc!A183 | `=LOG(-ABS(Data!C12),SUMPRODUCT(Data!A6:A6,Data!$B4,((-2.5)+2.5)))` | "#VALUE!" | "#NUM!" |
| gen-m5/1b30e4d075b252ea Calc!A365 | `=ISERR(SUMPRODUCT(Data!E6,123456789.123))` | true | false |
| gen-m5/399071ff6782ab16 Calc!A145 | `=SUMPRODUCT(TYPE(Data!B11:B11),Data!B9:E10,VLOOKUP((Data!C$2&"1.5"),Data!A9:C10,1))` | "#N/A" | "#VALUE!" |
| gen-m5/6a8bb7cd6d7769ad Calc!A79 | `=SUMPRODUCT(SUM(Data!E2:E11,COUNTIF(Data!B4:D5,"abc"),IFNA((-0.0001),1000000000000000.0)),Data!B7:...` | "#VALUE!" | 0.0 |
| gen-m5/7d1a5d223624fc70 Calc!A33 | `=SUMX2PY2(CELL("col",Data!$C11),IFERROR(#VALUE!,SUMPRODUCT(Data!B1,Data!C12,Data!D$10)))` | "#VALUE!" | 9.0 |
| gen-m5/aac6230c2fb40e1e Calc!A64 | `=MMULT(((1^Data!D11)^(Data!A3*Data!C3)),SUMPRODUCT(Data!A11:A11,1.005,Data!C11:C11))` | "#VALUE!" | 0.0 |
| gen-m5/aac6230c2fb40e1e Calc!A79 | `=SUMPRODUCT(AVERAGE(Data!E7:E11,Data!$C3,Data!D10:D12),Data!E5,((7-Data!E12)+IF(FALSE,Data!C4,Data...` | "#VALUE!" | 0.0 |
| gen-m5/f0f205865a3a4328 Calc!A396 | `=SHEET((7+SUMPRODUCT(Data!E1,100,(-1))))` | "#VALUE!" | "#N/A" |
| gen-m5/fc81c2f704bef138 Calc!A165 | `=SUMPRODUCT(ECMA.CEILING((Data!E4+123456789.123),Data!D5),Data!D4:D4)` | "#VALUE!" | 0.0 |

## Acceptance criteria
- [ ] These 9 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] coercion.md entry updated with the probe results and marked implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The SUMX family (T-0961).
