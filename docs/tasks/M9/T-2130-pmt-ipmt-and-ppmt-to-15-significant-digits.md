---
id: T-2130
title: "PMT, IPMT and PPMT to 15 significant digits on the shared power kernel, off powf"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
PMT, IPMT and PPMT are within half a unit of the 15th significant digit of Excel's value on
every sweep cell. They no longer use `powf`, and a bit-level fit is not required.

## Context
- `functions/financial/annuity.rs` `payment_growth` (around lines 41 and 141) still uses std
  `powf`. PV and FV already use `power(extended_sum(1.0, rate), nper)`, which matches all of
  their sweep cells (T-0901).
- Excel's PMT is within 3.5 ulp of the exact value (T-1528 context). On the literal cells of the
  2026-10-20 run, the engine's median relative error is 1.4e-15 and Excel's is 1.9e-16.
- Visibly-off cells on that run:
  - `probes-numeric/m9-fin-pmt`: PMT 81, IPMT 32, PPMT 137, e.g. `PMT(0.1,2,10000)` Excel
    -5761.904761904763, ours -5761.904761904758;
  - `ironcalc-calc/FINANCIAL/IPMT_PPMT.xlsx`: IPMT 7, PPMT 21.

  The RATE(n,PMT(..)) cells of `m9-fin-irr-rate` are RATE's (T-2005).
- T-1528 records that an accurate PMT (P−1 by expm1) fixed 30 Excel-cached cells but regressed
  22. Under decision 0083 such a trade is allowed only if each regressed cell stays within 4 ulp
  and none becomes visibly off. Check that first; if it does not hold, pick a formulation for
  which it does.
- The eight PV/NPER cells of decision 0064 should pass, or move closer, once PMT is accurate.
- Fit lab: `uv run rustsheet-fitlab annuity-pmt` (also `annuity-ipmt` and `annuity-ppmt`).

## Acceptance criteria
- [ ] Every PMT, IPMT and PPMT cell of m9-fin-pmt and IPMT_PPMT.xlsx is within half a unit of
      the 15th significant digit.
- [ ] Any cell that regresses follows decision 0083's allowance, and the Outcome lists it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity (T-1528, still time-boxed).
- NPV (T-2131 for IRR's NPV cells; T-1528 for NPV's last digits).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083).
