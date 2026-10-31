---
id: T-3485
title: "Probe whether ODDFYIELD is #NUM! when its Newton converges on the 100th check, like YIELD"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know whether ODDFYIELD, like YIELD, is `#NUM!` when its Newton's step meets the 1e-10 test on the 100th and last
check, and match it.

## Context
T-2747 found that YIELD is `#NUM!` exactly when `excel_yield_newton` (functions/financial/financial_bonds/yield_solver.rs)
meets its step test on check 100, and a value on check 99 or when no check passes (probes/yield-newton-step-cap;
financial.md "YIELD's #NUM! cells and its 100-step cap"). ODDFYIELD runs the same Newton (`odd_first_yield`) but
T-2747 applied the rule to YIELD only, for want of evidence. Find ODDFYIELD bonds whose Newton converges on checks 98
to 102 the way T-2747 did (a long climb back after a first step to a negative yield, at a huge coupon far below par;
trace the check count with a temporary uncapped build), and probe them with the excel-probe skill.

## Acceptance criteria
- [ ] A probe case gives Excel's ODDFYIELD on bonds that converge on checks 98 to 102 in our Newton.
- [ ] ODDFYIELD matches it (`NewtonStop::converged_on` in `odd_first_yield`), and financial.md says so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD (done in T-2747); ODDLYIELD (closed form or its own solver).
