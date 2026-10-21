---
id: T-2267
title: "TIME: an argument past 32767 is #NUM!, and a near-integer hour rounds before it truncates (probe the bound)"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
TIME is `#NUM!` when an hour, minute or second is past Excel's bound (probed; Microsoft documents 32767), and a near-integer hour such as 99.99999999999999 counts as the integer, as Excel gives.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/dates.md "TIME: an argument past 32767 is
`#NUM!`; a near-integer hour counts as that integer" (added by this triage). `fn_time` (crates/rustsheet-engine/src/functions/date_and_time.rs)
floors each argument and wraps any total into a day, so every cell is a number. The evidence only says the bound is in 23..44986
(minute 45306, hour 44986, seconds 230230 and more are `#NUM!`).
Probe first (excel-probe skill, excel-graph, about 30 cells): each of hour, minute, second at 32767, 32767.5, 32768, -32768, -32769 and
65535, 65536; `TIME(0,0,-1)`; `TIME(99.99999999999999,0,0)`, `TIME(23.9999999,0,0)`, `TIME(0,59.99999999,0)` and their HOUR/MINUTE, to fix
the rounding window. Record the rule in dates.md, then implement. Several CHOOSEROWS/ADDRESS cells need this too (T-2273, T-2274).
Why opus: the bound and the rounding window need a probe and judgement.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 34 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/04fdc9a67474aab9 Calc!A163 | `=TIME(1,Data!C12,COMBIN(26,6))` | "#NUM!" | 0.7056712962962961 |
| gen-m6/07171fa3f7493f51 Calc!A49 | `=TIME(9,DATEVALUE("1/15/2024"),39)` | "#NUM!" | 0.8379513888888894 |
| gen-m6/07171fa3f7493f51 Calc!A353 | `=TIME(SUM(Data!A9:A12),COUNTA(SUM(Data!E3,Data!D6),EOMONTH(47676,-3),SUM(Data!D5:D11,1000000000000000.0)),39)` | "#NUM!" | 0.6640625 |
| gen-m6/121a7c17092038d9 Calc!A140 | `=TIME(20,9,COMBINA((Data!E6+Data!B$7),6))` | "#NUM!" | 0.0 |
| gen-m6/144bceacd894868a Calc!A111 | `=TIME(123456789.123,34,SUMIF(Data!E5:E7,1))` | "#NUM!" | 0.8986111115664244 |
| gen-m6/216e3c05f407ca79 Calc!A270 | `=TIME(16,IF(ISNUMBER(0.5),123456789.123,Data!B6),57)` | "#NUM!" | 0.5485763888864312 |
| gen-m6/2ba5bd794e7e8987 Calc!A122 | `=TIME(9,38,SQRT(2^53-1))` | "#NUM!" | 0.853530092592564 |
| gen-m6/2ba5bd794e7e8987 Calc!A312 | `=TIME(1000000000000000.0,44,15)` | "#NUM!" | 0.6953125 |
| gen-m6/36d5ce763ea63631 Calc!A53 | `=TIME(Data!A11,36,((Data!$E6*Data!D4)+VALUE(Data!A7)))` | "#NUM!" | 0.7148146629333496 |
| gen-m6/36d5ce763ea63631 Calc!A129 | `=TIME(123456789.123,(Data!$A7*EOMONTH(59.999988426,10)),51)` | "#NUM!" | 0.8755902778357267 |
| gen-m6/36d5ce763ea63631 Calc!A319 | `=TIME(9,123456789.123,30)` | "#NUM!" | 0.25659722222189885 |
| gen-m6/44194b2fefa30de2 Calc!A213 | `=TIME(4,Data!B7,13)` | "#NUM!" | 0.611328125 |
| gen-m6/4ca39e0876796fa5 Calc!A304 | `=TIME(1000000000000000.0,9,COUNTIF(Data!D9:E12,"<>1"))` | "#NUM!" | 0.671875 |
| gen-m6/535caabe1e635e2d Calc!A151 | `=TIME(23,27,MAX(Data!B6:B12,SUBTOTAL(11,Data!B8:E11,Data!A4:C5,Data!B10:D10)))` | "#NUM!" | 0.0 |
| gen-m6/535caabe1e635e2d Calc!A379 | `=TIME(12,SUMPRODUCT(SUM(1e-15),Data!D10:D10),DATE(2076,12,(Data!$E9*Data!E10)))` | "#NUM!" | 0.6271527777777777 |
| gen-m6/6684de07ac74bcb8 Calc!A39 | `=TIME(22,29,123456789.123)` | "#NUM!" | 0.8348263888888141 |
| gen-m6/7ac4ea222977f3b4 Calc!A147 | `=TIME(11,9,(TRUNC(2^53+1,6)-IF(FALSE,Data!D$2)))` | "#NUM!" | 0.7816009521484375 |
| gen-m6/9615a6de0f3ac417 Calc!A236 | `=TIME(18,33,FACT(13))` | "#NUM!" | 0.7729166666686069 |
| gen-m6/9615a6de0f3ac417 Calc!A388 | `=TIME((Data!$D10+MIN(1,123456789.123)),25,SQRT(DAY(Data!D2)))` | "#NUM!" | 0.7265625 |
| gen-m6/9e6a1b733ee1e1e0 Calc!A77 | `=SECOND(TIME(1000000000000000.0,10,48))` | "#NUM!" | 30.0 |
| gen-m6/b54448af3fde07ca Calc!A47 | `=TIME(20,0.3,INT(2958466))` | "#NUM!" | 0.07483796296295964 |
| gen-m6/b62dd1ebd266b71c Calc!A260 | `=TIME(((123456789.123-(-0.5))+(1^Data!B9)),40,43)` | "#NUM!" | 0.9449421297758818 |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A220 | `=TIME((WORKDAY.INTL(34694,-14)*MROUND(1000000000000000.0,0.2)),Data!$D$10,21)` | "#NUM!" | 0.0 |
| gen-m6/bf63ed6c6f592475 Calc!A19 | `=TIME(12,EXP(PRODUCT(Data!E4,100)),48)` | "#NUM!" | 0.0 |
| gen-m6/c4edd76f46980c5f Calc!A13 | `=TIME(Data!B2,32,16)` | "#NUM!" | 0.8974074078723788 |
| gen-m6/c93ea588c271840e Calc!A102 | `=TIME(15,1,ROUNDUP(1E+21,6))` | "#NUM!" | 0.0 |
| gen-m6/c96ac1ada46179c4 Calc!A206 | `=TIME(6,(COUNTIF(Data!D1:D5,"<>")*CEILING(Data!D4,123456789.123)),6)` | "#NUM!" | 0.6563194444170222 |
| gen-m6/c96ac1ada46179c4 Calc!A244 | `=TIME(4,SUM(123456789.123,Data!B10:B10,42),59)` | "#NUM!" | 0.07915509259328246 |
| gen-m6/c96ac1ada46179c4 Calc!A320 | `=TIME(SUM(Data!C7:C11,Data!E10:E12,(Data!B6*100)),23,59)` | "#NUM!" | 0.640625 |
| gen-m6/d183b62596b8e174 Calc!A331 | `=TIME((COLUMNS(Data!A9)/POWER(0.1,Data!$B7)),12,35)` | 0.17540509259259274 | 0.13373842592592577 |
| gen-m6/d183b62596b8e174 Calc!A369 | `=TIME(6,(-1),MAX(Data!E1:E3))` | "#NUM!" | 0.1473263888888141 |
| gen-m6/de083028d8b676d2 Calc!A29 | `=TIME(23,(SUBTOTAL(7,Data!A1:D11,Data!D8:E12,Data!D4:E8)-(0*Data!D7)),28)` | "#NUM!" | 0.24407958984375 |
| gen-m6/ee56a4ed5a612a0d Calc!A251 | `=TIME(SUM(ROUND(2958466,Data!E$1)),32,7)` | "#NUM!" | 0.43896990740904585 |
| gen-m6/ee56a4ed5a612a0d Calc!A289 | `=TIME(DATEVALUE("2023-03-01"),27,26)` | "#NUM!" | 0.4357175925924821 |

## Acceptance criteria
- [ ] These 34 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] A new probes case (excel-graph) holds the bound grid; every cell passes or is a filed task named in the Outcome.
- [ ] dates.md states the bound and the rounding window with cells (no UNVERIFIED left for them); "TIME divides by 86400 first" still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TIMEVALUE and time text.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
