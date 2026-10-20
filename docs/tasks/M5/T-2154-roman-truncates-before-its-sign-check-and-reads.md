---
id: T-2154
title: "ROMAN truncates before its sign check and reads its form first; BASE of a negative fraction is #NUM!"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ROMAN truncates before its sign check and validates its form even for 0; BASE refuses a negative number (even one that truncates to 0) with `#NUM!`.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_roman` (arabic_roman.rs) returns "" for a number below 1 before reading `form`, and refuses -0.5025 before truncating it;
`fn_base` (mathematical.rs) truncates first, so `BASE(-0.0001,5,4)` is "0000". Rule: docs/excel-semantics/numbers.md "ROMAN
truncates before its sign check; BASE checks the sign first" (added by this triage).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 6 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/0362d48f56108faa Calc!A39 | `=ROMAN((FLOOR(Data!D11,Data!B1)*SUM(Data!$B$5)),VLOOKUP(COUNT(Data!C1:C11,Data!A$5),Data!E7:E11,3,...` | "#N/A" | "" |
| gen-m5/1962de140dae0111 Calc!A206 | `=ROMAN(0.2,LOG10(Data!E2))` | "#VALUE!" | "" |
| gen-m5/49bfbf67a1409563 Calc!A179 | `=ROMAN((IF(Data!B7,Data!D10,Data!D10)*1.005))` | "" | "#VALUE!" |
| gen-m5/a6fec430ac350c1a Calc!A31 | `=BASE(Data!$B$6,5,4)` | "#NUM!" | "0000" |
| gen-m5/c0651f0e0fb544d0 Calc!A253 | `=BASE(SUM((-0.0001),(Data!D11*Data!E1)),7,10)` | "#NUM!" | "0000000000" |
| gen-m5/d1b251fd6e9c4b4b Calc!A24 | `=BASE(AVERAGE((-0.0001)),9,1)` | "#NUM!" | "0" |

## Acceptance criteria
- [ ] These 6 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BASE/DECIMAL lifting (T-2159).
