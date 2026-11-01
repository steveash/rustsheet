---
id: T-3353
title: "XIRR on two-flow series over long spans loses accuracy (1E-8 to 1E-7 off)"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
XIRR on two-flow series over long spans returns Excel's rate, which is close to the math, instead of a value 1e-8 to
1e-7 off.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 6). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_xirr` in crates/rustsheet-engine/src/functions/financial/mod.rs and its solver. A two-flow series has the
  closed form (−v1/v0)^(365/days) − 1; check whether the engine's stopping tolerance or start leaves the iterate early.
- Excel's XIRR is about 1e-8 off the math in places (solver tolerance; decision 0188 R3): below G, so Excel's value is
  the target where the two differ, and the closed form is a reference, not the answer.
- Related: T-1583 and T-2328 (done) fitted XIRR's XNPV stop test and bound on long spans, and T-3287 (todo) probes the
  negative-root stop over spans of several years. The long-span error may be Excel's own stop tolerance, which those
  tasks reproduce: change the stop only against the new oracle cells, and keep what they reproduce.

## Probe cells (listed before any engine change)
- probes/xirr-two-flow-long-span (270 rows, excel-graph 2026-10-31): Calc!A = -1000, B = 1000·(1+r)^(days/365) to 6
  significant digits, C = 36526 (2000-01-01), D = C + round(years·365.25), years in {1, 2, 3, 5, 7, 10, 15, 20, 30, 40,
  50, 60, 75, 90, 100} (outer, 18 rows each), r in {-0.9, -0.5, -0.2, -0.05, -0.01, 0.001, 0.01, 0.03, 0.05, 0.08,
  0.12, 0.2, 0.35, 0.5, 1, 2, 5, 10} (inner). E = XIRR(A:B,C:D), F = the closed form in Excel's `^`, G = XNPV(E,...).
  The failing region of the assessed engine: the 49 negative-root cells E37, E55, E73, E75, E91-E93, E110-E113,
  E127, E129-E131, E145-E149, E163, E164, E166, E167, E181-E185, E199-E203, E217-E221, E235-E239, E253-E257. Every
  other cell is on both sides of it (positive roots at every span, and negative roots over 1-2 years).

## Acceptance criteria
- [ ] Two-flow XIRR over spans of 1 to 100 years and rates from -0.9 to 10 matches Excel's value within 15
      significant digits, with the right type and error, on the oracle-filled cells. The Gate A envelope below is only a
      floor on the fresh sweep; Excel's value is the target where it differs from the math by less than G = 1e-4.
- [ ] The Outcome separates engine error from reproduced Excel stop behaviour (cell by cell for the cells that move),
      and says how this task and T-3287 split the cells.
- [ ] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [ ] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [ ] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Outcome
- **No engine change: the bug was already fixed by T-3268 (#2441).** The owner's assessment ran at 921ed6ad, where
  XIRR's XNPV bound on a negative root was span-scaled (|XNPV| < 2e-8·max|v|·max(1, years), T-1583/T-2328). A replay
  of that rule misses 49 of the 270 probe cells. All 49 are negative roots over 3 to 100 years, stopping 2.3e-10 to
  2.8e-7 off Excel (item 6). T-3268 dropped the span factor before this task. Current main is bit-exact with Excel
  on all 810 probe cells (E, F, G), and on the 240 cells of the fresh sweep.
- **Engine error vs Excel stop behaviour, per cell.** No cell moves in this PR. On the 49 cells, the error at 921ed6ad
  was the engine's: a span-scaled bound Excel does not have. Today's residue against the math is all Excel's stop
  (the step test 5e-9 + 1e-8·|m|): about 1e-8 relative away from 0, 1e-7 to 2.4e-6 at roots near 1e-3, reproduced
  bit for bit.
- **Split with T-3287.** This probe settles T-3287's question (1) for two-flow series: the bound is unscaled, and its
  Context now says so. T-3287 keeps the three-flow and negative-guess cells, question (2) (midpoint sign or bracket
  side) and the financial.md doc fixes.
- **Gate A fresh sweep** (probes-numeric/xirr-two-flow-gate-a-sweep, 120 points, `random.Random(3353)`, log-uniform per
  stratum, 50-digit closed form committed as text in column H; filled excel-graph 2026-10-31). Each stratum below is
  counted as: points within 15 digits of Excel (all bit-equal), then points within 1e-9 relative of the math, then
  points outside both, then error/number mismatches.
  - neg-long (40): 40, 6, 0, 0.
  - neg-short (10): 10, 0, 0, 0.
  - pos-long (30): 30, 4, 0, 0.
  - pos-short (10): 10, 0, 0, 0.
  - near-zero (15): 15, 0, 0, 0.
  - extreme (15): 15, 7, 0, 0.

  Excel's median relative error against the math is 2.2e-8. 0 points are outside both envelopes, and no cell is an
  error on one side and a number on the other.
- **Found:** near a zero root (|root| below about 2e-5) Excel is 1e-4 to 2.1e-3 off the math, over G, on 9 of
  the 15 near-zero sweep cells (rows 94-104). That is an R3 candidate, filed as T-3484 (blocked on an owner ruling). The engine still matches Excel there.
- Eval delta: +2 cases, +1050 passing cells (probes 810, probes-numeric 240), 0 regressions. Docs: financial.md T-3353
  bullet.
