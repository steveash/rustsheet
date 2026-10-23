---
id: T-2033
title: "DB and DDB: Excel's declining-balance operation order (DB_DDB.xlsx, fin-argument-modes H88, H95)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
DB and DDB return Excel's bits by computing the depreciation in the operation order Excel uses, which
gap triage 2026-10-19 fitted on every Excel cell in the repo.

## Context
Known failures, all NumericUlp: ironcalc-calc/FINANCIAL/DB_DDB.xlsx Sheet1!G4, G5, G7, G10, G20, G31,
G35, G36, G37, G39, G42 (DB) and H10 (DDB) in eval/baseline/ironcalc-calc.txt (12 lines, Excel-cached);
probes/fin-argument-modes Sheet1!H88 `DB(10000,1000,6,2,7)` (Excel 2596.3941666666665, ours
...667) and H95 `DDB(10000,1000,6,2,2)` (Excel 2222.222222222222, ours 2222.2222222222217) in
eval/baseline/probes.txt (excel-graph). Example: G20 `DB(100,10,4,1,2)` is 7.299999999999999 in Excel, ours 7.3.

Fitted formulations (scratch Python over the cells, IEEE doubles with libm `pow`; they reproduce all
24 numeric DB cells and all 23 numeric DDB cells of DB_DDB.xlsx, including every cell that passes today,
plus H88 and H95). They rely on two checks the engine already makes ahead of the formula
(functions/financial/mod.rs): `month` is truncated (without it G10, G11, G17, G18 and G40 miss), and
cost 0 returns 0 (G30, G32-G34, outside the 24); keep both:
- DB: `rate = round(1 - (salvage/cost)^(1/life), 3)` as today; `first = cost * rate * (month/12)`
  (month over 12 first; ours `cost * rate * month / 12`); period 1 returns `first`. Then keep the
  remaining *balance*, not the accumulated depreciation: `balance = cost - first`, and for each period
  from 2 to period-1 `balance -= balance * rate`. A middle period returns `balance * rate`; the last
  one (period = trunc(life) + 1) returns `balance * (rate * (12 - month) / 12)` (rate times the months
  first). Ours accumulates `total += (cost - total) * rate` and returns `(cost - total) * rate`.
- DDB: `rate = factor / life` (1 when it reaches 1, as today); `old = cost * (1 - rate)^(period - 1)`
  as today; the depreciation is `min(old * rate, old - salvage)`, floored at 0. Ours is
  `old - cost * (1 - rate)^period` (switching to `old - salvage` below the salvage).
Code: `db_depreciation` and `ddb_depreciation` in
crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs. Keep the argument checks and
the loop bound as they are (T-1820 owns the bound and lifting). Record both formulations in
docs/excel-semantics/financial.md citing the cells above.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): DDB and DB values of the financial sweep, visibly off in the 13th to 15th digit (`DDB(13201,343,37,5,2)` Excel 571.3475689020747, ours ...761; at a life of 1E+9, `DDB(86243,423,1E+9,6,1)` 8.624299956878502E-05, ours 8.624300244264305E-05).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 36 known-failure lines in eval/baseline/gen-m9.txt, 36 visibly off.
Examples:
- gen-m9/5c6a23c4f6597f1c Calc!A355 `=DDB(59706,388,38,1)`: Excel 3142.4210526315787, ours 3142.421052631573
- gen-m9/5ded7594e597e31e Calc!A3 `=DDB(96556,203,22,3)`: Excel 7254.395191585273, ours 7254.395191585281
- gen-m9/6376e27be7a05871 Calc!A337 `=DDB(91340,201,17,2)`: Excel 9481.66089965398, ours 9481.660899653987
- gen-m9/b72d952c5f52cb4d Calc!A304 `=DDB(5832,692,25,1)`: Excel 466.56, ours 466.5599999999995
- gen-m9/c0681a8b150ce301 Calc!A226 `=DDB(80593,570,35,4)`: Excel 3860.0858189087876, ours 3860.085818908781
- gen-m9/d912671baa1364aa Calc!A163 `=DDB(82793,233,39,3)`: Excel 3821.4945295773696, ours 3821.4945295773796
Cells:
- gen-m9/0a957119475760c5.case.json (2): Calc!A176, A323
- gen-m9/1b60bf99d71424e7.case.json (1): Calc!A343
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A121, A366
- gen-m9/30e20ba3302fea8b.case.json (3): Calc!A22, A120, A365
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A101
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A83
- gen-m9/5422bdbf173fc233.case.json (3): Calc!A106, A253, A302
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A355
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A3
- gen-m9/6376e27be7a05871.case.json (2): Calc!A141, A337
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A126
- gen-m9/79d87fc527927626.case.json (1): Calc!A51
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A111
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A253, A351
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A290
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A170
- gen-m9/abb7d565af89b849.case.json (1): Calc!A129
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A163
- gen-m9/b72d952c5f52cb4d.case.json (2): Calc!A304, A353
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A273
- gen-m9/c0681a8b150ce301.case.json (2): Calc!A79, A226
- gen-m9/d0854366830dcdee.case.json (1): Calc!A209
- gen-m9/d912671baa1364aa.case.json (2): Calc!A16, A163
- gen-m9/e168be7f6d3f84c2.case.json (2): Calc!A19, A21

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 102 known-failure lines in eval/baseline/gen-m9.txt, 102 last-digit (not visibly off).
Examples:
- gen-m9/0a957119475760c5 Calc!A345 `=DB(28432,1,19,4)`: Excel 2349.3576170333276, ours 2349.3576170333286
- gen-m9/3aa168259335be54 Calc!A31 `=DDB(5569,59,25,1)`: Excel 445.52, ours 445.5199999999995
- gen-m9/5ded7594e597e31e Calc!A344 `=DB(85976,1,38,3)`: Excel 12212.504939712, ours 12212.504939712002
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A345
- gen-m9/1b60bf99d71424e7.case.json (5): Calc!A98, A147, A245, A294, A392
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A88, A317
- gen-m9/1dce4774f5c0e736.case.json (4): Calc!A5, A299, A317, A348
- gen-m9/30e20ba3302fea8b.case.json (5): Calc!A62, A71, A160, A267, A316
- gen-m9/31474c20fc1dafd6.case.json (6): Calc!A134, A150, A183, A199, A297, A346
- gen-m9/3aa168259335be54.case.json (4): Calc!A31, A178, A325, A329
- gen-m9/3c06f9298f65a3ba.case.json (2): Calc!A230, A377
- gen-m9/5422bdbf173fc233.case.json (3): Calc!A91, A155, A189
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A304
- gen-m9/5ded7594e597e31e.case.json (4): Calc!A199, A248, A344, A346
- gen-m9/6376e27be7a05871.case.json (5): Calc!A7, A43, A105, A288, A386
- gen-m9/67a4c03afd63f988.case.json (5): Calc!A175, A213, A224, A262, A371
- gen-m9/762cd0e4ec9a7aa9.case.json (3): Calc!A4, A91, A298
- gen-m9/79d87fc527927626.case.json (6): Calc!A2, A98, A296, A343, A392, A394
- gen-m9/7d2979fba2c95d64.case.json (4): Calc!A62, A160, A258, A307
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A155, A302
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A241
- gen-m9/9a1fa5366bc50918.case.json (3): Calc!A83, A145, A328
- gen-m9/9d9ceba257f8001b.case.json (4): Calc!A56, A133, A154, A378
- gen-m9/a91bad4e84f49739.case.json (3): Calc!A72, A227, A374
- gen-m9/abb7d565af89b849.case.json (5): Calc!A126, A224, A273, A276, A325
- gen-m9/b6a02893a8d6b151.case.json (2): Calc!A212, A370
- gen-m9/b72d952c5f52cb4d.case.json (3): Calc!A10, A205, A206
- gen-m9/b73c6cc96e093f03.case.json (2): Calc!A102, A168
- gen-m9/ba4ddb817a8e700e.case.json (3): Calc!A77, A322, A371
- gen-m9/bc04c1f478c9896c.case.json (4): Calc!A85, A297, A330, A395
- gen-m9/c0681a8b150ce301.case.json (4): Calc!A128, A177, A324, A363
- gen-m9/d0854366830dcdee.case.json (1): Calc!A160
- gen-m9/d912671baa1364aa.case.json (2): Calc!A114, A209
- gen-m9/e168be7f6d3f84c2.case.json (3): Calc!A68, A119, A362

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 36 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 102 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
- [ ] The 12 DB_DDB.xlsx lines leave eval/baseline/ironcalc-calc.txt and fin-argument-modes H88 and H95
      leave eval/baseline/probes.txt (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests for G20, G42 (`DB(23,10,4,5,2)` = 1.8687313963962309, the last-period branch), H10
      (`DDB(100,10,4,3,1.3334)` = 14.814814787037497) and H95.
- [ ] The financial.md entry cites the cells; `make check` green; reviewers approve.

## Out of scope
- VDB (T-1975, T-1977, T-1978); lifting and the period-loop bound (T-1820).
- If a cell still differs after both changes, stop and hand the task up (work-loop step 5b) instead
  of trying other operation orders.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.
