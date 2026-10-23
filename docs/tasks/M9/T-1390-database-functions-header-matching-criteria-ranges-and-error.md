---
id: T-1390
title: "Database functions: header matching, criteria ranges and error choice differ from Excel on generated data (gen-m9 database sample)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The database functions match Excel on the shapes that generated workbooks produce: header matching
with non-text or blank headers, criteria ranges that overlap the database or have one row, and the
error each one returns.

## Context
A seeded sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`, using a scratch catalog that
held only the database signatures from T-1086) was not committed. excel-graph accepted all 20
workbooks, and the engine passed only 673 of 800 cells. Examples (Excel, then ours):
- `DSUM(Data!D1:E8,2,Data!C3:E8)` is 1000000000000003.6, where we give 0.
- `DVARP(Data!A1:D11,4,Data!B3:C11)` is 11.9025, where we give `#DIV/0!`.
- `DAVERAGE(Data!A7:A9,1,Data!B9:B9)` is `#VALUE!` (one-row criteria), where we give 3.14159.
- `DGET(Data!A1:D10,1,Data!C1:E4)` is "?b", where we give `#NUM!`.
- `DMAX(Data!A1:A10,1,Data!C3:D8)` is 123456789.123, where we give 0.
- `DAVERAGE(Data!A10:D12,4,Data!B3:E7)` is `#DIV/0!`, where we give -1.25005.
- `DCOUNT(Data!B4:D6,0,Data!D9:D10)` is 0, where we give `#VALUE!`.
The likely root causes are how a criteria header that is a number, a boolean or blank matches a
database column, criteria rows that are entirely blank, and when a one-row criteria range is
`#VALUE!`. Write a probe with one question per cell (excel-probe), record the rules in
docs/excel-semantics/statistics.md, then fix. Split the task if it is bigger than M. T-1054 covers
DVAR/DSTDEV numerics.

gen-m9 sweep (gap triage 2026-10-23, T-2255). 46 cells where a database function inside a statistical call is the
innermost divergence (DAVERAGE, DGET, DSUM, DSTDEV, DSTDEVP, DVAR, DVARP, DCOUNT, DCOUNTA, DMIN, DMAX, DPRODUCT over the
cases' Data sheet; the enclosing call only passes the value on, so the cell's Excel value is the D-function's or follows
from it). T-2258 (the database sweep) will add direct evidence.
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 46 known-failure lines in eval/baseline/gen-m9.txt, 46 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A246 `=BETADIST(Data!A9,0.01,10000000000.5,DVARP(Data!A11:D12,4,Data!B5:D6),(MODE(Data!C8:C10,Data!C7:C11)-2.5))`: Excel #DIV/0!, ours #VALUE!
- gen-m9/27601819397dd56e Calc!A265 `=F.INV.RT(DSTDEV(Data!B9:E12,3,Data!D8:D10),1.5,1)`: Excel #NUM!, ours #DIV/0!
- gen-m9/285d87cea1aaa73c Calc!A210 `=HYPGEOM.DIST(5,DVARP(Data!C9:C10,1,Data!A7:C7),9,9,Data!$E8)`: Excel #VALUE!, ours 1.0
- gen-m9/2a18b2f5a4dc43ac Calc!A334 `=LINEST(IF(ISREF(Data!A2:B10),MIN(Data!C12,Data!C1:C3,(-0.5)),(Data!B5-1e-15)),((0+Data!D10)*DMIN(Data!A2:D11,4,Data!B9:D12)))`: Excel 0.0, ours #VALUE!
- gen-m9/530c636749a85209 Calc!A191 `=LOGEST(SUM(DAVERAGE(Data!C8:C10,4,Data!A8:D9),Data!A$2),0.5,(FACT(59)>=NORMSINV(2.5)),TRUE)`: Excel #VALUE!, ours #NUM!
- gen-m9/58e3253bbd3a7a82 Calc!A100 `=GAMMAINV(0.001,((0/(-1))+0.2),T.DIST.2T(DSTDEVP(Data!B9:C10,1,Data!B8:C11),6))`: Excel #DIV/0!, ours #NUM!
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A246
- gen-m9/27601819397dd56e.case.json (1): Calc!A265
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A210
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A334
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A279
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A345
- gen-m9/4250f880ab640c49.case.json (1): Calc!A118
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A335
- gen-m9/530c636749a85209.case.json (2): Calc!A174, A191
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A308
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A100, A265
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A374
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A25, A324
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A329
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A122
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A64, A89, A235
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A154, A347
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A217
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A297
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A328
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A66, A261
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A328
- gen-m9/c18dbf21f11093f3.case.json (4): Calc!A24, A178, A254, A351
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A43
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A210
- gen-m9/cde551ea787f713a.case.json (1): Calc!A13
- gen-m9/d8266bc15533702a.case.json (1): Calc!A343
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A372
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A318
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A221
- gen-m9/f2bd5f380ec002bf.case.json (4): Calc!A40, A154, A321, A345
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A313
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A288

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 46 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe answers each root cause, and the rules are in docs/excel-semantics/.
- [ ] The probed cells pass, and a regenerated seed-17 database sample passes at least 95% of cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting (T-1389).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.
