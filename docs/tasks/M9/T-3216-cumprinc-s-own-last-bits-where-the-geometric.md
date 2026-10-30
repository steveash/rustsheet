---
id: T-3216
title: "CUMPRINC's own last bits where v^nper does not underflow: 87 principal cells and 4 CUMIPMT cells on them, 1 to 2 ulps off (fin-cumprinc-form-boundaries, fin-cumipmt-gate-edges, fin-cumipmt-period-bound)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3179, T-3118]
defer: phase-1b
---

## Goal
CUMPRINC returns Excel's bits on the principal cells that T-3067 left 1 to 2 ulps off, where `v^nper` does not
underflow. Where no form fits, financial.md "T-3067 notes" lists the forms tried with their counts on these cells.

## Context
Gap triage 2026-10-30 (`make eval` on origin/main): these are failing cells that no open task names. Every one is
within 15 digits (NumericUlp), so the task is deferred to Phase 1b (decision 0114). It needs numeric fitting, so the
tier is opus.

T-3067 run 3 moved the principal's form cut to the leading discount `(nper-end+1)·L` (`GEOMETRIC_MIN_LEAD`,
`LONG_GEOMETRIC_MIN_LEAD` in functions/financial/cumulative.rs). That left every cell below visibly right, but 59 of
the 132 form-boundary cells are 1 to 2 ulps off (financial.md "T-3067 notes": "Where the sum changes form").
Run 4 found that with Excel's own PMT bits plugged in, the best type 1 principal form (the geometric sum
`pmt·((1-v^m)/(r/(1+r)))·v^(nper-end+1)` on `log1p` or `growth_log`, with Excel's expm1) is exact on only 22 of 58
cells. So CUMPRINC's own operation order is still open, and T-3067's outcome hands its last digits on. T-1374 does
not name these cells, and T-3141 owns only the underflowed principals.

Cells (excel-graph; `make eval-show CASE=probes/<case>.case.json`):
- probes/fin-cumprinc-form-boundaries (59): A7, A11, B2, B5, B7, B11, B12, C2, C4, C5, C7, C8, C10, C12, C14, D1, D2,
  D5, D7, D10, D12, D14, E6, E8, F4, G3, G6, G7, G8, H1, H3, H6, H7, H8, I1, I5, I6, I8, I9, I10, J1, J2, J3, J6, J8,
  J9, J10, K5, K6, K7, K11, L1, L3, L5, L6, L7, L8, L9, L10
- probes/fin-cumipmt-gate-edges (25): C3, C4, C7, C9, C10, D3, D4, D7, D9, D10, F3, F4, F5, F6, F7, F8, F9, F10, G1,
  G3, G5, G6, G7, G8, G10
- probes/fin-cumipmt-period-bound (6): A8, A12 (CUMPRINC); A3, A5, A7, A23 (CUMIPMT, which T-3118 builds as
  `PMT·m - CUMPRINC`, so they follow these bits once T-3118 lands)
- probes/fin-cumipmt-overflow-and-forms (1): A6

Examples (Excel vs ours):
- form-boundaries B2 `CUMPRINC(2,600,319288,3,42,0)`: -1.8643286040007455E-261 vs -1.8643286040007453E-261
  (a long range that ends at a deep discount: the geometric sum on `growth_log`).
- form-boundaries J1 `CUMPRINC(0.1,1100,478581,7,1100,0)`: -478581.0 vs -478581.00000000006 (a long range that ends at
  nper, below the long-range cut).
- gate-edges F5 `CUMPRINC(1E-5,100,100000,3,10,1)`: -7996.400496822623 vs -7996.40049682262 (a tiny rate, type 1).
- period-bound A8 `CUMPRINC(0.0001,5000,1000,1,5000,0)`: Excel exactly -1000.0, ours -1000.0000000000001 (the whole
  loan; also A12 at type 1, ours -999.9999999999999).

The whole-loan cells (period-bound A8, A12; form-boundaries J1, I1) show that Excel's sum gives back exactly `-pv`
at `start = 1, end = nper`. That is a cheap first test for any candidate form.

Code: `cumulative.rs` (principal forms and their cuts); `annuity.rs` (`discounted_payment`, PMT's `1-Q`, T-3014). Read
docs/research/excel-kernel-playbook.md first; decision 0056 applies (two attempts that do not raise the bit-exact
count, then time-box).

## Acceptance criteria
- [ ] Each of the 91 cells above passes, or financial.md "T-3067 notes" lists every candidate form tried with its
      bit-exact count on these four cases.
- [ ] No new known-failure lines in any baseline (`make eval-baseline` records the fixed ones). In particular,
      CUMPRINC_CUMIPMT.xlsx and the fin-cumipmt-* and fin-cumprinc-* cases lose no passing cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Principals where `v^nper` underflows: fin-cumprinc-near-1e15, fin-cumprinc-underflow-ends (T-3141).
- CUMIPMT's own construction (T-3118) and the `1-Q` switch (T-3179).
