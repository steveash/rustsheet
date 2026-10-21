---
id: T-2048
title: "ISPMT last ulp: Excel's operation order (ISPMT.xlsx E2)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ISPMT matches Excel to the last bit, from an operation order that fits every ISPMT cell with Excel
values, so ironcalc-calc/FINANCIAL/ISPMT.xlsx passes in full.

## Context
- Known failure (gap triage 2026-10-19): ironcalc-calc/FINANCIAL/ISPMT.xlsx Sheet1!E2
  `ISPMT(0.2,1,3,100)`: Excel (Macintosh cache) -13.333333333333334, ours -13.333333333333336. The
  workbook's own F2 `D2*A2*(B2/C2-1)` gives ...336 too, which is the engine's
  `pv * rate * (per / nper - 1.0)` (`ispmt_of`, crates/rustsheet-engine/src/functions/financial/annuity.rs).
  This cell is not among the desktop/web disagreements in docs/research/oracle-calibration.md, so
  Excel for the web agrees.
- Triage check on the workbook's 7 numeric ISPMT rows (E2, E3, E8, E10, E13, E14, E15; inputs
  rate, per, nper, pv in A:D): the engine's form fits 6 (not E2); `pv*rate*(per-nper)/nper` fits 6
  (E2 but not E15); `pv*rate/nper*(per-nper)` and `pv*(rate*(per-nper))/nper` fit all 7. Two
  candidates tie, so add oracle cells (excel-probe skill, excel-graph) that separate them (search
  random inputs where the two differ) and check the ISPMT cells of the probes suite
  (fin-argument-modes and the lifting probes) before choosing.

## Acceptance criteria
- [x] The known-failure line `ironcalc-calc/FINANCIAL/ISPMT.xlsx Sheet1!E2` leaves
      eval/baseline/ironcalc-calc.txt with no new lines in any suite.
- [x] docs/excel-semantics/financial.md records ISPMT's order with the cells that decide it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other annuity functions (T-1528).

## Outcome
Probed 24 ISPMT cells where the candidate orders differ in the last bit (probes/fin-ispmt-order, excel-graph): only
`(per - nper) * pv * rate / nper` fits all 24 (the old `pv*rate*(per/nper-1)` fits 8; the two tied candidates from
triage fit 6 and 17). `ispmt_of` uses it; financial.md records "ISPMT's operation order".
Eval: ironcalc-calc known failures 1056 -> 1055 (ISPMT.xlsx Sheet1!E2), +24 new passing probe cells, no regressions; the
existing ISPMT probe cells (fin-argument-modes, fin-later-argument-ranges) still pass.
