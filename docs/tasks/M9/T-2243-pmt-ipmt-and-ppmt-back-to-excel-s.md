---
id: T-2243
title: "PMT, IPMT and PPMT back to Excel's bits on the 14 cells T-2130 moved by 1 to 4 ulp"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-2130]
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
