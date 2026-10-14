---
id: T-1528
title: "YIELD with one coupon period left: use the closed form (bit-exact on probes/solver-iteration-traces Y!A, Y!B)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1525]
---

## Goal
YIELD returns Excel's bits when at most one coupon period is left before maturity.

## Context
T-1525 found that Excel does not iterate in this case. It evaluates the documented closed form
`((red/100 + rate/f) - (pr/100 + A/E*rate/f)) / (pr/100 + (A/E)*(rate/f)) * (f*E/DSR)` in that
order of operations, with DSR the settlement-to-maturity day count of the basis (DAYS360 on basis 0:
180 for 2024-03-01 to 2024-08-31), not E - A. That form is bit-exact on all 20 cells of
`probes/solver-iteration-traces` Y!A1:A10 (semiannual, 2024-03-01 to 2024-06-30) and Y!B1:B10 (annual,
2024-03-01 to 2024-08-31). See `docs/excel-semantics/financial.md`, "YIELD with one coupon period left
is closed form", and `docs/research/t1525-solver-fit.py` (`score_yield_one_period`).
The engine iterates instead (`fn_yield` in
`crates/rustsheet-engine/src/functions/financial/financial_bonds.rs`) and misses the 20 cells by 1 to 15 ulp.

## Acceptance criteria
- [ ] YIELD with one coupon period or less left uses the closed form, in Excel's order of operations.
- [ ] probes/solver-iteration-traces Y!A1:A10 and Y!B1:B10 pass; no other cell regresses (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD with more than one coupon period (its solver is unfitted: T-1525 Outcome, T-1374).
