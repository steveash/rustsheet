---
id: T-2659
title: "Probe ODDFPRICE and ODDLPRICE at a yield where the discount power overflows"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ODDFPRICE and ODDLPRICE match Excel at a yield where `(1+yld/frequency)^t` overflows (a number or `#NUM!`), established by a probe.

## Context
T-2488 found that PRICE, DURATION and MDURATION are `#NUM!` once `(1+yld/frequency)^t` overflows at the redemption's
term (financial.md "PRICE, DURATION and MDURATION at a huge yield or coupon", probes/bond-huge-yield-overflow). ODDFPRICE
and ODDLPRICE are listed there as unprobed. Ours discounts with `discount_growth` (financial_bonds.rs), whose overflow
gives an infinite growth and a zero flow, so an overflowing yield prices as a number. Write a probe case (excel-probe
skill) bracketing the overflow yield for ODDFPRICE (`OddFirstSchedule::price`) and ODDLPRICE (linear in the yield, so
probably never overflowing; check it with yields up to 9.99999999999999E+307, the largest literal Excel accepts), record
the rule in financial.md, and match it.

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with excel-graph values brackets the boundary for ODDFPRICE and ODDLPRICE.
- [ ] The rule is in financial.md and every probe cell passes (or is a recorded known failure with an owner).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD and ODDFYIELD (T-2391).
