---
id: T-2033
title: "DB and DDB: Excel's declining-balance operation order (DB_DDB.xlsx, fin-argument-modes H88, H95)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
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

## Acceptance criteria
- [ ] The 12 DB_DDB.xlsx lines leave eval/baseline/ironcalc-calc.txt and fin-argument-modes H88 and H95
      leave eval/baseline/probes.txt (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests for G20, G42 (`DB(23,10,4,5,2)` = 1.8687313963962309, the last-period branch), H10
      (`DDB(100,10,4,3,1.3334)` = 14.814814787037497) and H95.
- [ ] The financial.md entry cites the cells; `make check` green; reviewers approve.

## Out of scope
- VDB (T-1975, T-1977, T-1978); lifting and the period-loop bound (T-1820).
- If a cell still differs after both changes, stop and hand the task up (work-loop step 5b) instead
  of trying other operation orders.
