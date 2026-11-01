---
id: T-2667
title: "IPMT's cancelling pv-0 cells at rates 10 and 0.1: Excel's own Y to the last bit (probes, 7 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IPMT's cancelling pv-0 cells have Excel's value to 15 digits: the 7 visibly-off cells of
probes/fin-ipmt-ppmt-huge-rate (A195, A196, A203, A204, A211, A212, B35).

## Context
T-2490 (docs/excel-semantics/financial.md "IPMT cancels on fv") found that where IPMT's balance cancels (`|B| < |fv|`), Excel
computes `B = (pv+fv)*Y - fv` in doubles on a rounded `Y`, so IPMT carries fv's rounding. `balance_interest`
(crates/rustsheet-engine/src/functions/financial/annuity.rs) rounds the exact `Y`. That gives 590 of the probe's 595 IPMT
cells to 15 digits. The cells listed in the goal depend on `Y`'s last bit, which Excel computes some other way:
`IPMT(10,2,10,0,315)` is 1.2144607808295405E-6 in Excel and 1.214461349263729E-6 in ours; `IPMT(10,5,10,0,315)` and
`IPMT(10,10,10,0,315)` are one ulp of 315 off in B; `IPMT(0.1,2,360,0,1)` is 0 in Excel and 1.11E-17 in ours. Each also has a
type-1 cell. Candidates: `Y` from Excel's EXP and the `growth_log` L, with `1-Q` taken directly or through expm1 (both
fitted worse globally in T-2490: 592 and 586 of 595); P-based quotients with Excel's `^`; a 64-bit Y. Probe a
dense pv-0 grid at rates 2, 10 and 0.1 with nper 5 to 400 before choosing, and keep every m9-fin-pmt cell passing.
The `|B| < |fv|` switch is fitted (0.1 to 1 fit), not an identified formulation: re-fit it on the new grid.

## Acceptance criteria
- [x] The 7 cells of the goal pass to 15 digits, with no new known-failure lines in any baseline.
- [x] financial.md "IPMT cancels on fv" states the fitted rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- Past-range PPMT and fractional nper (T-2665, T-2666).

## Outcome
Done by T-2243. IPMT now splits Excel's discounted PMT (docs/excel-semantics/financial.md "IPMT and PPMT split Excel's discounted payment"), one form bit-exact on all 1495 IPMT cells with constant arguments, so no fitted switch is needed. The 7 pv-0 cells pass to the bit; `IPMT(10,2,10,0,315)` is 1.2144607808295405E-6.
