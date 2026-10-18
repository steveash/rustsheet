---
id: T-1842
title: "YIELD and the ODD* bond functions lift over arrays, with their solver and coupon work weighted (SPEC-0011)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1822]
---

## Goal
YIELD, ODDFPRICE, ODDFYIELD, ODDLPRICE and ODDLYIELD lift over arrays in every argument and refuse a multi-cell range there, with each element's solver and coupon work charged to the lifted call.

## Context
Split from T-1822, which entered PRICE, DURATION, MDURATION, ACCRINT and the COUP* family as `LiftArrays` weighted per coupon
schedule walk (functions/lift/kinds.rs `COUPON_WALK_WEIGHT`, financial.md "Coupon schedule work cap"). YIELD's solver
(`compute_yield_from_price`) runs up to 100 Newton steps of three PRICE evaluations and 200 bisection steps of two, each
PRICE a walk and a sum over up to about 32,400 coupons, so one element can cost seconds at the calendar's longest
schedule; ODDFYIELD and ODDLYIELD solve over ODDFPRICE/ODDLPRICE alike (`solve_yield`), and ODDFPRICE walks its schedule
(`coup_number`). Decide each kernel's weight from a measured worst case (or a tighter per-call bound first), then enter
them. Every argument is probed over an array (fin-argument-modes H190-H230, H344-H351) and over a range (H191-H223, H345;
fin-remaining-later-argument-ranges H105-H164, H173-H184, all `#VALUE!`). Their settlements read through
`get_settlement_and_maturity` / `get_odd_period_dates`, which T-1822 left node-based for YIELD and the ODD* functions.

## Acceptance criteria
- [ ] The YIELD and ODD* lift cells of probes/fin-argument-modes pass (baseline lines removed), no new failures.
- [ ] A test shows a lifted YIELD over many elements is refused by the work bound before its first element.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The solver stopping points (T-1374, T-1525).
