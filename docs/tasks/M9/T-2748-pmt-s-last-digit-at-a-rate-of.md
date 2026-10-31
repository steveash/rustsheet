---
id: T-2748
title: "PMT's last digit at a rate of 1E-15 with type 1 (PMT(1E-15,346,96301,42,1)), visibly off through COSH in gen-m9"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PMT at a tiny rate with payments at the start of the period gives Excel's last digit, so the gen-m9 cell that amplifies
it through COSH matches within 15 digits.

## Context
Split from T-2620.
- probes/t2475-stat-subexpressions P!A462: `PMT(1E-15,346,96301,42,1)` is -278.44797687866065 in Excel (excel-graph,
  2026-10-24); ours is -278.4479768786607, one ulp off.
- gen-m9/bcb1105c61a2649d Calc!A97: `STDEVP(Data!E3:E12,Data!A6,COSH(PMT(Data!C9,346,96301,42,1)))`, with Data!C9 1E-15.
  Excel gives 1.5802420409865438e120 and ours 1.580242040986634e120. COSH makes the one ulp visible.

At 1E-15 the `(1+r)^n - 1` term is all cancellation. Which form Excel takes there (expm1 of n·ln(1+r), the double
`^`, or the rate-0 branch's -(pv+fv)/n with the type-1 factor) decides the last bit. T-2730 (phase-1b) fits PMT's
general last digits over the T-2242 sweeps. This cell is visibly off, so it cannot wait for phase 1b. Probe PMT for
rates 1E-16 to 1E-12 at both types (excel-probe skill), fit `compute_payment` (functions/financial/annuity.rs), and
record the form in financial.md "Annuity evaluation order".

## Acceptance criteria
- [ ] P!A462 and gen-m9/bcb1105c61a2649d A97 leave the baselines (`make eval-baseline`), with no new known failures.
- [ ] The tiny-rate form is in docs/excel-semantics/financial.md with its probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT's general last digits over the T-2242 sweeps (T-2730).
