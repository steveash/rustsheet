---
id: T-2025
title: "DB and DDB last digits: fit Excel's depreciation loop order (DB_DDB.xlsx)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2023]
---

## Goal
DB and DDB return Excel's bits on the cached workbook and the sweep, through a fitted evaluation order
of their period loops.

## Context
Gap triage 2026-10-19. All misses are 1-2 ulps (NumericUlp):
- eval/baseline/ironcalc-calc.txt, `ironcalc-calc/FINANCIAL/DB_DDB.xlsx` Sheet1: DB G4, G5, G7, G10, G20,
  G31, G35, G36, G37, G39, G42 (e.g. G4 Excel 21.920191799999998, ours 21.9201918; G20 7.299999999999999 /
  7.3; G31 -0.12966666666666665 / ...668) and DDB H10 (14.814814787037497 / ...498).
- eval/baseline/probes.txt, probes/fin-argument-modes Sheet1!H88 `DB(10000,1000,6,2,7)`
  (2596.3941666666665 / ...667) and H95 `DDB(10000,1000,6,2,2)` (2222.222222222222 / ...2217). Both are
  also in T-1374's list; this task takes them over.
Excel's DB rounds the rate to 3 decimals, takes `cost*rate*month/12` in the first period and
`(cost - total)*rate` after, and a last-period rule with `(12-month)/12`; the misses suggest a different
association (for example `cost*rate*month/12` vs `cost*(rate*month)/12`, accumulating `total` vs
recomputing `cost - total`). Code: `fn_db` (crates/rustsheet-engine/src/functions/financial/mod.rs),
`fn_ddb` (financial_depreciation.rs); VDB shares DDB's step and has its own ulp task (T-1975): keep one
definition of the DDB step and check VDB's cells do not regress.
**Needs an oracle sweep:** fit against `probes-numeric/m9-db-ddb-sweep` (T-2023), never blind; time-boxed
(decision 0056): record every candidate's bit-exact and within-4-ulp counts.

## Acceptance criteria
- [ ] The 14 lines above leave eval/baseline/ironcalc-calc.txt and eval/baseline/probes.txt, and the
      sweep's cells pass, or the Outcome lists the residue with the candidate table.
- [ ] No new F lines in any baseline (VDB included).
- [ ] The fitted order is recorded in docs/excel-semantics/financial.md with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DB's and VDB's lifting (T-1820) and VDB's own last digits (T-1975).
