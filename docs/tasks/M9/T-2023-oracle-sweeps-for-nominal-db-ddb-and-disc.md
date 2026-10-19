---
id: T-2023
title: "Oracle sweeps for NOMINAL, DB, DDB and DISC last-digit fits"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle-filled probes-numeric cases give the NOMINAL (T-2024), DB/DDB (T-2025) and DISC (T-2026) fit
tasks enough Excel points to compare candidate formulations in the fit lab, as decision 0056 requires
("every fit is a fit-lab comparison of named candidates against an oracle sweep").

## Context
Gap triage 2026-10-19: the failing cells of these functions are a few cached Mac-Excel values
(ironcalc-calc NOMINAL_EFFECT.xlsx, DB_DDB.xlsx, DISC.xlsx) plus three probe cells; the closed forms are
short, so the misses are evaluation-order differences that need a grid to separate. No sweep of these
functions exists in eval/suites/probes-numeric (checked). Excel for the web agrees with those caches
(docs/research/oracle-calibration.md lists no disagreement for them), so web values are the target.
One case per family, written with the excel-probe skill and `rustsheet-oracle fill` (excel-graph,
canary-verified), each one formula per cell, plain numbers in, no lifting:
- `m9-nominal-sweep`: `NOMINAL(e,n)` for about 20 rates in [0.0001, 5] (include 0.5, 0.21939108490523185,
  2125.8224907304752 and the NOMINAL_EFFECT.xlsx effective rates) by npery 1, 2, 3, 4, 6, 7, 10, 12, 52,
  100, 365, 1000 (about 240 cells).
- `m9-db-ddb-sweep`: `DB(cost,salvage,life,period,month)` over the DB_DDB.xlsx argument rows and a grid
  (cost 1000/10000/12345.67, salvage ratios 0.01-0.5, life 3/5/6/10, every period up to life+1, month
  1/6/7/12), and `DDB(cost,salvage,life,period,factor)` with factor 1.5/2/3 (about 400 cells).
- `m9-disc-sweep`: `DISC(settlement,maturity,pr,redemption,basis)` for bases 0-4 over about 12 date spans
  (month ends, 29 February, spans of 1 to 400 days), pr 90-99.9 and redemption 100/105 (about 300 cells).

## Acceptance criteria
- [ ] The three cases are committed under eval/suites/probes-numeric with oracle sidecars and no
      pending cells.
- [ ] The case names are the ones T-2024, T-2025 and T-2026 cite (or those task files are updated to
      the names used).
- [ ] Any quirk the sweeps show (an error where a value was expected, a period or basis rule) goes into
      docs/excel-semantics/financial.md with its cells.
- [ ] If the oracle is unavailable, this task is set `status: blocked` with the reason.
- [ ] `make check` green (new cells may fail as new coverage; no regressions); reviewers approve.

## Out of scope
- Fitting or changing any function (the dependent tasks do that); EFFECT (T-2022, already settled).
