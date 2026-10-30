---
id: T-3179
title: "PPMT, IPMT and CUMIPMT's 1-Q: does Excel switch from expm1 to 1-EXP(x) at |nper*L| 1 as PMT does"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3014]
---

## Goal
PPMT, IPMT, CUMIPMT and CUMPRINC use Excel's own `1-Q` (`1 - EXP(-nper*L)`) wherever Excel does, so their bits move
toward Excel's on the cells where Kahan's expm1 at |nper*L| >= 1 is the miss, with no cell regressing.

## Context
- T-3014 found that Excel's PMT takes `1-Q` from Kahan's expm1 only for |x| < 1 (x = -nper*L) and from
  `1 - EXP(x)` from 1 on (probes/t3014-pmt-fv-type-alone column C; financial.md "PMT is a discounted payment";
  numbers.md "Excel's expm1 is Kahan's formula"). Whether the switch is Excel's expm1 itself (and so PPMT's and
  IPMT's discounts too) is not established: SINH and TANH call expm1 only below 1.
- Scratch experiment (T-3014, 2026-10-29, not committed): the same switch in `discounted_principal`'s `periods`
  (crates/rustsheet-engine/src/functions/financial/annuity.rs) fixes 4 more cells of probes-numeric/m9-fin-pmt
  (721 -> 725 of 924) and 3 of gen-m9, but regresses 3 cells of ironcalc-calc (and fixes 3 others there). So PPMT's
  switch is not simply PMT's: the regressions are the lead (which workbook, which |x|).
- Evidence to score on: probes-numeric/m9-fin-pmt (IPMT, PPMT), ppmt-log-rate-sweep, ppmt-negative-rate-sweep,
  ironcalc-calc FINANCIAL/IPMT_PPMT.xlsx and CUMPRINC_CUMIPMT.xlsx, the probes fin-cumipmt-* cases. T-2243's
  remaining six cells (IPMT_PPMT.xlsx G12, G14, H13, H31, H4; probes/fin-cumipmt-overflow-and-forms A13) are
  candidates. Probe |x| around 1 for PPMT the way t3014-pmt-fv-type-alone column C does for PMT if the sweeps
  cannot place the switch.

## Acceptance criteria
- [ ] PPMT's (and IPMT's, CUMIPMT's, CUMPRINC's where they share it) `1-Q` follows the evidence: either the switch
      at |x| = 1 with the ironcalc-calc regressions explained and fixed, or a recorded finding that Excel's PPMT
      keeps Kahan's expm1 past 1 (with the cells that show it) in financial.md.
- [ ] No new known-failure line in any baseline; `make eval-baseline` records the fixed ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT (T-3014, done).
- RATE and NPV.

## Scope after T-2243 (2026-10-29)
IPMT and PPMT take PMT's `1-Q` switch: T-2243 found that they split Excel's discounted PMT, switch included, bit-exact on all 1495 IPMT cells and on 2552 of 2556 PPMT cells (financial.md "IPMT and PPMT split Excel's discounted payment"). What is left here is CUMIPMT and CUMPRINC. CUMIPMT's type-1 per-period sum still takes the exact split (`SplitForm::Exact`). Summing the discounted IPMT instead fixes 4 CUMPRINC_CUMIPMT.xlsx type-1 cells but regresses G3, G4 and G15, and fin-cumipmt-overflow-and-forms A8, by an ulp each. So CUMIPMT has a form of its own. T-2243 also hands this task its last cell: fin-cumipmt-overflow-and-forms A13 (`CUMIPMT(0.0083,36,8000,3,3,0)`, type 0, 3 ulp).
