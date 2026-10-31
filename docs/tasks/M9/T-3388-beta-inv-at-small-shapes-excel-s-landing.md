---
id: T-3388
title: "BETA.INV at small shapes: Excel's landing point where its cdf is flat or its search stops off the crossing (12 visibly-off m9-beta-inv cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3387]
campaign: visible
---

## Goal
BETA.INV and BETAINV return Excel's value, within half a unit of the 15th significant digit, at small and moderate
shapes where Excel's own BETA.DIST is flat around the quantile or where Excel's search stops before the crossing. The
model of Excel's start and stop is fitted under decision 0188 R4, or the task records the hypotheses ruled out.

## Context
- Gap triage 2026-10-31 (`make eval` on a64f963e, excel-graph evidence): `make exit-owners MILESTONE=M9` lists 20
  `rate:BETA.INV` cells of probes-numeric/m9-beta-inv with no open owner. The 5 `#NUM!` cells at p = 0.9999999999
  are T-3351's; 3 more sit where Excel's x is a crossing of its cdf and ours is not (T-2511's). This task owns the
  other 12. Excel vs ours, and the relative error of each against a 50-digit mpmath root (scratch, not committed):
  - flat (Excel's BETA.DIST is one double at x and both neighbours, so x is where the search stops):
    - A8 `BETA.INV(1E-10,0.5,1)`: 9.999999999999992E-21 vs 1.0000000000000028E-20 (Excel -8.0E-16, ours +2.8E-15)
    - A15 `BETA.INV(1E-10,0.5,2.5)`: 3.4697827972579854E-21 vs 3.4697827972579726E-21 (+2.2E-15, -1.5E-15)
    - A22 `BETA.INV(1E-10,0.5,10)`: 8.052722492439177E-22 vs 8.0527224924392055E-22 (-9.7E-16, +2.6E-15)
    - A23 `BETA.INV(0.001,0.5,10)`: 8.052726383222091E-8 vs 8.0527263832221E-8 (-5.0E-16, +6.1E-16)
    - A29 `BETA.INV(1E-10,0.5,100)`: 7.873641070203278E-23 vs 7.87364107020325E-23 (+2.6E-15, -9.7E-16)
    - A30 `BETA.INV(0.001,0.5,100)`: 7.873645161824792E-9 vs 7.873645161824812E-9 (-4.8E-16, +2.1E-15)
    - A33 `BETA.INV(0.95,0.5,100)`: 0.0190710957512068 vs 0.019071095751206746 (+5.5E-16, -2.3E-15)
    - A34 `BETA.INV(0.999,0.5,100)`: 0.052826250505828454 vs 0.0528262505058284 (-2.3E-16, -1.3E-15)
    - A43 `BETA.INV(1E-10,1,1)`: 9.999999999999996E-11 vs 1.0000000000000015E-10 (-4.0E-16, +1.5E-15; the uniform
      quantile is exactly 1E-10)
  - miss (x is off Excel's own crossing):
    - A94 `BETA.INV(0.05,2.5,10)`: 0.05199525015806432 vs 0.051995250158064235 (+2.0E-15, +3.9E-16)
    - A101 `BETA.INV(0.05,2.5,100)`: 0.005668759702401242 vs 0.0056687597024011795 (+1.3E-14, +1.6E-15)
    - A137 `BETA.INV(0.5,10,100)`: 0.08843223023126127 vs 0.08843223023126134 (-5.6E-16, +2.3E-16)
  Every point is in Gate A (both sides within 1E-9 of the math), so this is visible-parity work, not accuracy: at the
  miss cells Excel is the inaccurate side, and at the flat cells neither is correctly rounded.
- The same points are rows 7, 14, 21, 22, 28, 29, 32, 33, 41 (flat) and 91, 98, 133 (miss) of
  probes-numeric/t1745-beta-inv-roundtrip, which T-2511 owns as a whole; at rows 7, 14, 21, 32, 33 and 41 our
  BETA.DIST already equals Excel's at x and both neighbours, so the gap is in the search, not the cdf.
- Rule and quirk: docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)" (the
  22 flat and 11 off-crossing rows; "a tolerance stop, but it is not established").
- Hypotheses to test first: T-2555 reads Excel's BETA.INV as AS 109's structure (p > 1/2 is 1 - BETA.INV(1-p,b,a)
  bit for bit) and its tiny-quantile values as near the second Newton iterate from AS 109's start; the flat cells
  (Excel within 1E-15 of the root where its cdf has no resolution) fit a start such as (p·a·B(a,b))^(1/a) that the
  Newton steps cannot move. Share the start and stop model with T-2555 rather than writing a second search.
- Code: `fn_beta_inv` in crates/rustsheet-engine/src/functions/statistical/beta.rs, inverse_beta.rs (statrs' search
  and the T-2119 refinement to the crossing, `beta_quantile_crossing`), `special::refine_increasing`.
- Needs an oracle sweep: the 33 informative rows cannot separate start and stop hypotheses with R4.1's 30% hold-out,
  so this task depends on T-3387, which fills one.

## Acceptance criteria
- [ ] probes-numeric/m9-beta-inv A8, A15, A22, A23, A29, A30, A33, A34, A43, A94, A101, A137 are within half a unit of
      the 15th significant digit of Excel's value, or each one left is recorded in statistics.md as UNVERIFIED with the
      hypotheses ruled out and their bit-exact counts (decision 0056 time box).
- [ ] The fit follows decision 0188 R4: a deterministic ~30% hold-out of the T-3387 points with train and test exact
      rates side by side in the Outcome; the model beats the correctly rounded crossing on test; every fitted constant
      or cut carries `/// FITTED(T-3388, <n> cells, <range>)` and outside that range the code falls back to the
      accurate crossing.
- [ ] After the model is frozen, a fresh log-uniform probes-numeric sweep over BETA.INV's valid domain (filled,
      scored against 50-digit mpmath) is recorded in the Outcome: per stratum, points within 15 digits of Excel and
      points within 1E-9 relative of the math, with 0 outside both and 0 error-against-number cells (R4.2, Gate A).
- [ ] The probes-numeric t1745-beta-inv-roundtrip J column and m9-legacy-betainv lose no 15-digit cell; no gated cell
      regresses (`make eval-baseline` refuses regressions).
- [ ] statistics.md's T-1745 section states Excel's start and stop rule as found (or the ruled-out list).
- [ ] `make check` green; reviewers approve (reviewer-excel-fidelity sees the hold-out counts and the Gate A result).

## Out of scope
- p near 1 where the answer is 1 (T-3351); huge shapes (T-2555); moving the search onto the ported cdf (T-2511);
  the crossing cells whose miss comes from BETA.DIST's last bits (m9-beta-inv A31, A102, A103, T-2511); bit-level
  residue on cells already within 15 digits (Phase 1b, decision 0188 R5: file it with `defer: phase-1b`).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when every cell this task owns is within half a unit of the 15th
significant digit of Excel's value with the right type and error, or is an UNVERIFIED entry with its ruled-out
hypotheses. Bit-level residue moves to a follow-up with `defer: phase-1b`.
