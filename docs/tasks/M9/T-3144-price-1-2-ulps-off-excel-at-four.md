---
id: T-3144
title: "PRICE 1-2 ulps off Excel at four exact yields where the lab's ^ model is bit-exact (yield-price-at-iterates B4, C3, C7, D4)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PRICE matches Excel bit for bit on probes/yield-price-at-iterates B4, C3, C7 and D4, as the fit lab's model of
Excel's `^` already does.

## Context
Found by T-2974 (2026-10-29). probes/yield-price-at-iterates asks Excel's PRICE at exact yields written as
`(hi*67108864+lo)/2^k`. The lab's PRICE, discounted with `kernels/power.py`'s `excel_model` (`solvers.caret_pow`),
matches Excel on all 59 PRICE cells. The engine's PRICE (`CouponSchedule::price` on `discount_growth`, i.e.
`arithmetic::power`) misses four by 1 to 2 ulps:
- B4 `PRICE(36665,44491,1,(129855283*67108864+24469045)/2^53,100,4,0)`: Excel 102.79950206522773, ours ...774.
- C3 `PRICE(DATE(2096,3,1),DATE(2126,6,30),0.08,(96218381*67108864+3322353)/2^56,100.5,1,0)`: Excel 90.03029573044918,
  ours ...917.
- C7 `PRICE(DATE(2096,3,1),DATE(2099,6,30),0.05,(124704984*67108864+42041808)/2^57,100.5,1,0)`: Excel
  98.00117602302839, ours ...838.
- D4 `PRICE(36665,44491,1,(125972570*67108864+25266213)/2^53,100,4,0)`: Excel 106.00009537723355, ours ...358.
So the engine's `power` and the lab's `excel_model` disagree on at least one discount factor of each. First, find the
factor (base `1 + y/f`, exponent `k - 1 + DSC/E`) where they differ, then decide which piece of numbers.md
"Non-integer exponents: exp(y·ln x)" (the twice-rounded `*`, `excel_ln`, `excel_exp`, the `1/` of a negative
exponent) the engine computes differently. The fix belongs in the shared `^` kernel, not in PRICE.
Semantics: financial.md "PRICE and ODDFPRICE discount with Excel's `^`" and "YIELD stops short of PRICE's root"
(T-2974 paragraph). Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs `discount_growth`,
`arithmetic::power`.

## Acceptance criteria
- [ ] The four cells pass bit-exact (their lines leave eval/baseline/probes.txt) with no new known-failure line in any
      suite (`make eval-baseline`), or the task records the differing factor and the forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD's own last digits (T-2974, T-2788): Excel's YIELD iterates differ from ours, not its PRICE.
