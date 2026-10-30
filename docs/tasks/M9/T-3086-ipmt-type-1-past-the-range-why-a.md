---
id: T-3086
title: "IPMT type 1 past the range: why a whole rate keeps the exact quotient, and rate 1000's extra ulp"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: []
---

## Goal
IPMT type 1 past `(1+rate)^nper`'s range follows one identified Excel formulation instead of T-2665's fitted switch on whether the rate is a whole number, and the 48 rate-1000 cells of probes/t2665-ipmt-fractional-nper-past-range match.

## Context
T-2665 found that, past the range and where the balance does not cancel, Excel's type-1 IPMT is `-B*(rate/(1+rate))` on the rounded balance for 0.01, 0.1 and 1.005 (gen-m9 0a957119475760c5 A153), but the exact `-rate*B/(1+rate)` for rates 2 and 42 (fin-ipmt-ppmt-huge-rate A186, A190; gen-m9 b73c6cc96e093f03 A268). At rate 1000 neither fits: `IPMT(1000,7,1E+5,1000,0,1)` is -999.0009990009992 (exact ...990, the double form ...991). `balance_interest` in crates/rustsheet-engine/src/functions/financial/annuity.rs switches on `rate.fract()`, a fitted rule. `-(rate*B)*EXP(-LN(1+rate))` with libm fits rate 1000 but not 1.005; Excel's own EXP/LN (x87_exp.rs, the atanh-series L of financial.md "PPMT discounts with an atanh-series log") is untried. docs/excel-semantics/financial.md "IPMT cancels on fv".

## Acceptance criteria
- [x] One formulation, with Excel's EXP/LN where it uses them, fits every type-1 cell of probes/t2665-ipmt-fractional-nper-past-range and the past-range type-1 cells of fin-ipmt-ppmt-huge-rate it now passes, or a probe (e.g. a rate sweep of 2..50 and 0.9..1.1) shows why not; the `rate.fract()` switch goes or is explained.
- [x] No new known-failure lines.
- [x] `make check` green; reviewers approve.

## Out of scope
- IPMT's cancelling pv-0 cells (T-2667).

## Outcome
Done by T-2243. IPMT now splits Excel's discounted PMT (docs/excel-semantics/financial.md "IPMT and PPMT split Excel's discounted payment"), one form bit-exact on all 1495 IPMT cells with constant arguments, so no fitted switch is needed. All 600 IPMT cells of probes/t2665-ipmt-fractional-nper-past-range pass to the bit, rate 1000 type 1 included (`IPMT(1000,7,1E+5,1000,0,1)` is -999.0009990009992), as do the past-range type-1 cells of fin-ipmt-ppmt-huge-rate.
