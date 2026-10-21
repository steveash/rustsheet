---
id: T-2130
title: "PMT, IPMT and PPMT to 15 significant digits on the shared power kernel, off powf"
milestone: M9
lane: functions-financial
status: done
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
      the 15th significant digit. (All but four PPMT cells, where Excel itself is 7-41 ulp from the
      exact value: moved to T-2242.)
- [x] Any cell that regresses follows decision 0083's allowance, and the Outcome lists it.
- [x] `make check` green; reviewers approve.

## Out of scope
- Bit parity (T-1528, still time-boxed).
- NPV (T-2131 for IRR's NPV cells; T-1528 for NPV's last digits).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083).

## Outcome
- PMT, IPMT and PPMT compute the exact payment in double-double arithmetic: `accurate_growth`
  (square-and-multiply from the exact `1+rate`, times `1 + expm1(f*ln_1p(rate))` for a fractional
  exponent) and closed forms that do not cancel (`accurate_payment`, `accurate_period_split`). The
  double forms remain as the fallback past |P| ≈ 1E±270 or |nper| > 2^32 (the overflow edges of
  probes/pmt-overflow), with `powf` replaced by Excel's `^`. IPMT's `FutureValueForm::IpmtRecurrence`
  is gone (the T-1528 TODO).
- Visible parity: m9-fin-pmt visibly-off cells 250 → 4 (PMT 81 → 0, IPMT 32 → 0, PPMT 137 → 4);
  IPMT_PPMT.xlsx 28 → 0. The four left (A82, A166, A412, A418) are period 1 of long terms, where
  Excel is 7-41 ulp from the exact value; T-2242 has the lead (`PMT*exp((per-1-nper)*log1p(rate))`
  fits 153 of the 154 fv-0 type-0 PPMT cells).
- Eval: 71 known failures fixed (ironcalc-calc 903 → 855, probes 1600 → 1590; m9-fin-pmt bit-exact
  258 → 445 of 924); in-scope visibly-off cells −54 (1716 → 1662 when measured, 1758 → 1704 on the
  `main` it merged; the lock itself is left to the steward, decision 0093).
- Regressed under decision 0083's standing allowance (numeric, ≤ 4 ulp, none visibly off; listed
  in T-2243): IPMT_PPMT.xlsx G12, G14, H13, H31, H4 (1 ulp each); PMT.xlsx F15, F33, G10 (1 ulp),
  and the PV/FV cells on them J15 (2), J33 (1), K10 (1), M10 (1), L15 (4).
  After `main` (with T-1890's CUMPRINC closed form) merged in, three more under the same allowance:
  CUMPRINC_CUMIPMT.xlsx H15 (1 ulp), H17 (2) and probes/fin-cumipmt-overflow-and-forms A13 (3),
  also listed in T-2243.
- The eight PV/NPER cells of decision 0064 did not move to passing: they take Excel's cached PMT
  bits, which are not the exact payment.
- Follow-ups: T-2242 (the four PPMT cells, campaign visible), T-2243 (the 13 one-ulp cells).
