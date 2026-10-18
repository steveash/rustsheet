---
id: T-1870
title: "YIELD and the other coupon-bond functions: a negative basis is #NUM!, not basis 0"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1842]
---

## Goal
`YIELD(39448,43831,0.0575,95.04,100,2,-1E300)` is `#NUM!`, as Excel gives it (probes/fin-yield-odd-lift L5),
and the other coupon-bond functions treat a negative basis the way Excel does.

## Context
The bond kernels read the basis as `floor() as u32` (functions/financial/financial_bonds.rs `bond_args`,
`coupon_args`, `odd_args`), which saturates a negative number to 0, so a negative basis is computed as basis 0
(ours 0.063467439 for the L5 cell). The discount-security functions are `#NUM!` at basis -1
(financial.md "Discount-security functions"). Probe first: basis -1, -0.5 and -1E300 for YIELD, PRICE,
DURATION, a COUP* function, ACCRINT and one ODD* function (whether -0.5 truncates to 0 is open), then make
the reads check the number before the cast. A negative frequency is already `#NUM!` (it saturates to 0).

## Acceptance criteria
- [ ] fin-yield-odd-lift L5 passes (baseline line removed) and the new probe cells pass.
- [ ] financial.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other argument reads of these functions.
