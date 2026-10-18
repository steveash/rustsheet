---
id: T-1823
title: "Financial compute helpers build no message for lifted error elements; bench RATE at the cap"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1372]
---

## Goal
A lifted financial call's error elements allocate nothing, and the lifted and per-row paths of
the financial functions have measured numbers.

## Context
T-1372's performance review found that the shared helpers in functions/financial/annuity.rs
(`compute_payment`, `compute_future_value`, `compute_ipmt`, `annuity_power` via `formula_error`)
and financial_util.rs `compute_rate` still build a message String per error element
("Invalid result", "Failed to converge", IPMT/PPMT's `format!("Period must be between 1 and {}")`),
which `calc_result_to_array_node` drops at once. SPEC-0011 allows one allocation per element, so
this is not a bug, only waste. RATE's `Weighted(50)` never refuses under the 2^20 output cap; its
worst case (about 2^20 × 50 Newton steps) is estimated, not measured.

## Acceptance criteria
- [ ] The `compute_*` helpers return the error kind (or a `&'static str` message) and lifted
  elements use empty messages; scalar results and error kinds are unchanged.
- [ ] benches/lift.rs gains a non-converging `=SUM(RATE(A1:A100000,0,100,1))` case and a per-row
  unlifted `=PMT(A{r}/100,10,100)` over ≥ 100k rows (main vs branch); the numbers go in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DB/VDB/CUMIPMT loops (T-1820, T-1822).
