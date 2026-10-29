---
id: T-2243
title: "PMT, IPMT and PPMT back to Excel's bits on the 14 cells T-2130 moved by 1 to 4 ulp"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-2130, T-3014]
campaign: visible
---

## Goal
The 14 Excel-cached cells that T-2130 moved from bit-exact to within 4 ulp are bit-exact again,
with no cell made visibly off.

## Context
- T-2130 computes PMT, IPMT and PPMT as the exact payment (double-double, `accurate_growth` in
  `functions/financial/annuity.rs`). That fixed 71 cells and took 54 visibly-off cells off the
  lock, but Excel is up to 3.5 ulp from the exact value, so 14 cells that the old double forms
  matched by luck are now 1 to 4 ulp off (decision 0083's allowance; T-2130 Outcome):
  - IPMT_PPMT.xlsx G12, G14, H13, H31, H4 (IPMT, 1 ulp each);
  - PMT.xlsx F15, F33, G10 (PMT, 1 ulp), and J15 (2), J33, K10, M10 (1) and L15 (4), which are
    PV/FV of those PMT cells.
  - probes/fin-cumipmt-overflow-and-forms A13 (CUMIPMT over one period, 3 ulp): after `main`'s T-1890
    merged in, CUMIPMT's short per-period interest sum takes `period_payments` (CUMPRINC never does).
- T-1528 holds the time-boxed bit-level fit of PMT; this task is its regression list.

## Findings (2026-10-28, opus run; why this waits on T-3014)
The visibly-off cells are PMT's last bits amplified, so this task is PMT's bit fit and cannot land before it:
- `PMT.xlsx` J10 `PV(-0.9,3,F10,300,1)`: dPV/dpmt = -(1+r)((1+r)^3-1)/(r(1+r)^3), about -111 at rate -0.9, so F10's
  4-ulp gap (Excel -2703.783783783783; the exact value on the double rate, -2703.78378378378444..., rounds to
  ours, -2703.7837837837847) becomes
  2.2E-10 (Excel 119.99999999977724, ours 120.00000000000462). The same holds for J11, J18, J23, H10, H11
  (rows 10, 11, 18, 23: rate -0.9, type 1).
- Excel's PMT on those four rows is 3 to 6 ulp *below* the exact magnitude (F10 -4, F11 -6, F18 and F23 -3
  ulp from the unrounded exact value). No form scored reproduces them: `-(fv+pv*P)*r/(k*(P-1))` and 14 algebraic rearrangements
  with `P` by `pow` or `exp(n*log1p(r))`; T-3014's discount forms (`docs/research/t3014_pmt_discount_score.py`:
  `-(pv+fv*Q)/(k*((1-Q)/r))` misses F10 by -3, F11 -4, F18 -3, F23 -3); and `1+r` snapped to 0.1 in `P`
  and/or `k` (still -1 to -6). Over PMT.xlsx's 48 numeric PMT cells with a nonzero rate (F and G columns)
  the best of these, `P` by `exp(n*log1p(r))` and `-(pv*r/(1-1/P)+fv*r/(P-1))/k`, is bit-exact on 37; the
  discount forms score 17-20 of the 38 with a positive integer nper.
- These scores are from ad-hoc scratch scripts (not committed), each form evaluated in Python doubles on the
  cached arguments; T-3014's fit should re-score them with Excel's EXP/LN models.
- So the rate -0.9 type-1 rows are the hardest discriminator for T-3014's fit: score candidate forms on
  them first.

## Acceptance criteria
- [ ] The 14 cells are bit-exact, and no cell of m9-fin-pmt, IPMT_PPMT.xlsx, PMT.xlsx or
      CUMPRINC_CUMIPMT.xlsx regresses or becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The four PPMT cells Excel computes inexactly (T-2242).

## Deferred (decision 0114), superseded
This task was `defer: phase-1b` on the belief that all its cells were within 15 digits. The 2026-10-27 eval showed
some of them visibly off, so T-0090 removed the key and gave it a `campaign:` stage (below), as this section foresaw.

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value. Bit-level criteria still unmet then move to a follow-up filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a): this task was `defer: phase-1b`, but it is the only
live owner of visibly-off ironcalc-calc cells, so the key is removed (decision 0114 parks only tasks whose cells are
all within 15 digits): FINANCIAL/PMT.xlsx Sheet1!J10, J11, J18, J23 (PV, e.g. 119.99999999977724 expected,
120.00000000000462 today), H10, H11, H27, I27 (NPER), L32, L33 (FV), and FINANCIAL/CUMPRINC_CUMIPMT.xlsx Sheet1!G2
(CUMIPMT). Fixing them lifts PV, NPER and FV to the Phase 1 rate. T-2730 names the same cells and stays parked
for its PMT bit work.

## After T-3014 (2026-10-29)
T-3014's PMT (financial.md "PMT is a discounted payment") makes the whole of PMT.xlsx pass, so the eight PMT.xlsx
cells here (F15, F33, G10, J15, J33, K10, L15, M10) are fixed. Left: IPMT_PPMT.xlsx G12, G14, H13, H31, H4 and
probes/fin-cumipmt-overflow-and-forms A13, all IPMT/PPMT, which still start from the exact payment; T-3179 tries
PMT's `1-Q` switch there.
