---
id: T-3486
title: "YIELD's 100th iterate on a negative-yield climb drifts up to 2.5e-12 from Excel's (9 yield-below-par-sweep cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
YIELD's value where its Newton runs all 100 steps on the climb back from a negative yield matches Excel to 15
significant digits.

## Context
Found by T-2747 (probes/yield-below-par-sweep, `YIELD(39681,45662,R,P,102,4)`; financial.md "YIELD's #NUM! cells and
its 100-step cap"). Where the first step from the fitted start jumps to a negative yield, Newton climbs back by small
steps and, past 100 steps, returns the 100th iterate, which is no root (negative, or just above 0 as at D11). Ours
matches Excel in type there, but 8 cells are NumericClose, up to 2.5e-12 relative: Y!A10, B10, C11, D11, D12, E12,
F13, G15 (D11: Excel 0.11011934742871272, ours 0.11011934742844205). H17 is within 1 ulp. (The sweep's ninth
NumericClose cell, Y!B1, is a converged root about 5 ulp off: T-2788's.) The drift builds up over the steps, so a small difference in
PRICE's or its slope's evaluation at yields where 1 + y/4 is small (huge powers in `discount_growth`, or the
summation order in `CouponSchedule::price` / `dprice`) may be the cause. T-2788 owns the last ulps at the root,
not these.

## Acceptance criteria
- [ ] The cause is found (for example, compare the iterates with Excel's at fewer steps by probing `PRICE` at our
      iterates) and the 9 cells match to 15 digits, or financial.md records why they cannot.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The step cap and the check-100 `#NUM!` (T-2747), the last ulps at the root (T-2788).
