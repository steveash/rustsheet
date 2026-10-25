---
id: T-2243
title: "PMT, IPMT and PPMT back to Excel's bits on the 14 cells T-2130 moved by 1 to 4 ulp"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2130]
defer: phase-1b
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

## Acceptance criteria
- [ ] The 14 cells are bit-exact, and no cell of m9-fin-pmt, IPMT_PPMT.xlsx, PMT.xlsx or
      CUMPRINC_CUMIPMT.xlsx regresses or becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The four PPMT cells Excel computes inexactly (T-2242).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
