---
id: T-2243
title: "PMT, IPMT and PPMT back to Excel's bits on the 13 cells T-2130 moved by an ulp"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2130]
---

## Goal
The 13 Excel-cached cells that T-2130 moved from bit-exact to within 4 ulp are bit-exact again,
with no cell made visibly off.

## Context
- T-2130 computes PMT, IPMT and PPMT as the exact payment (double-double, `accurate_growth` in
  `functions/financial/annuity.rs`). That fixed 71 cells and took 54 visibly-off cells off the
  lock, but Excel is up to 3.5 ulp from the exact value, so 13 cells that the old double forms
  matched by luck are now 1 to 4 ulp off (decision 0083's allowance; T-2130 Outcome):
  - IPMT_PPMT.xlsx G12, G14, H13, H31, H4 (IPMT, 1 ulp each);
  - PMT.xlsx F15, F33, G10 (PMT, 1 ulp), and J15 (2), J33, K10, M10 (1) and L15 (4), which are
    PV/FV of those PMT cells.
- T-1528 holds the time-boxed bit-level fit of PMT; this task is its regression list.

## Acceptance criteria
- [ ] The 13 cells are bit-exact, and no cell of m9-fin-pmt, IPMT_PPMT.xlsx, PMT.xlsx or
      CUMPRINC_CUMIPMT.xlsx regresses or becomes visibly off.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The four PPMT cells Excel computes inexactly (T-2242).
