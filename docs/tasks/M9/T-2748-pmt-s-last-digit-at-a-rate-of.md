---
id: T-2748
title: "PMT's last digit at a rate of 1E-15 with type 1 (PMT(1E-15,346,96301,42,1)), visibly off through COSH in gen-m9"
milestone: M9
lane: functions-financial
status: done
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
- [x] P!A462 and gen-m9/bcb1105c61a2649d A97 leave the baselines (`make eval-baseline`), with no new known failures.
- [x] The tiny-rate form is in docs/excel-semantics/financial.md with its probe cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- PMT's general last digits over the T-2242 sweeps (T-2730).

## Outcome
Both cells had already left the baselines on origin/main when this task started (2026-10-31): PMT's discounted
payment (T-3014) gives `PMT(1E-15,346,96301,42,1)` = -278.44797687866065 to the bit, and gen-m9/bcb1105c61a2649d
A97 follows. New evidence: probes/pmt-tiny-rate (160 cells, rates 1E-16 to 1E-12, both types, five nper/pv/fv sets)
and probes/pmt-tiny-rate-near-2e-16 (30 cells) show that form holds at every tiny rate except 2^-53 to 2^-52, where
Excel is 1 to 3 ulp larger in magnitude (20 cells). Taking `2+rate` in the x87 register in `growth_log` was tried and
regressed 294 cells (fixing 23), so it is not the cause. That band is T-3489 (opus). Recorded in financial.md "PMT is a discounted
payment". No code change. Eval delta: probes +190 checked, +170 passing, 20 new known failures, all in the two new
cases and listed in T-3489.
