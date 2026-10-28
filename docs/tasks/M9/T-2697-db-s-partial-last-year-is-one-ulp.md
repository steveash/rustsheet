---
id: T-2697
title: "DB's partial last year is one ulp off (DB(0.01,488,32,33,10)) while DB_DDB G42 fixes the grouped order"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
DB's partial last year (period life+1, month below 12) matches Excel to the bit in both known cells.

## Context
Found by T-2491. probes/fin-syd-db-domain C1 and C2 (`DB(0.01,488,32,33,10)`, `DB(0.01,488,32,33.5,10)`, excel-graph) are
-30.88768801325266 in Excel; ours -30.887688013252664 (one ulp). The left-to-right last-year charge
`balance * rate * (12 - month) / 12` gives Excel's C1 but regresses ironcalc-calc FINANCIAL/DB_DDB.xlsx G42
(`DB(23,10,4,5,2)`, Excel 1.8687313963962309, which the current grouped `balance * (rate * (12 - month) / 12)` matches;
financial.md "DB and DDB operation order", T-2033). So neither order is Excel's: the balance walk itself may differ (a
running total rather than a running balance, or `(1 - rate)` products), or the rate's rounding. Probe more last-year cells
(rate negative and positive, several months, short and long walks) and fit one order that passes both.
Code: `db_depreciation` in crates/rustsheet-engine/src/functions/financial/financial_depreciation.rs.

## Acceptance criteria
- [ ] One operation order passes probes/fin-syd-db-domain C1:C2 and every DB cell of DB_DDB.xlsx and
      probes/fin-argument-modes; the C1/C2 lines leave eval/baseline/probes.txt (`make eval-baseline`), no new lines.
- [ ] financial.md "DB and DDB operation order" states the fitted order with the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DDB.
