---
id: T-2972
title: "PRICE, DURATION and MDURATION bit-exact residue at a huge coupon rate and yield after T-2733 (47 cells of probes/bond-extreme-coupon-yield)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2733]
defer: phase-1b
---

## Goal
PRICE, DURATION and MDURATION give Excel's bits on probes/bond-extreme-coupon-yield, not only its 15 digits.

## Context
T-2733 found the sum order that brings every cell of the case within 15 significant digits (financial.md "PRICE,
DURATION and MDURATION sum order at a huge coupon rate and yield"): 196 of 239 PRICE cells are bit-exact. 47 cells of
the case stay last-digit failures (see eval/baseline/probes.txt once T-2733 lands), plus gen-m9/0a957119475760c5
Calc!A279. The lead the fit left: PRICE's redemption term alone (rate 0, Parts!B3, B5:
`PRICE(38136,44737,0,42,100,2,0)` 3.0121596882492917e-47) is not `100/((1+42/2)^(DSC/E+N-1))` with the worksheet `^`
(Parts!D3 is that power to the bit, and 100/D3 is 3.012159688249292e-47), so PRICE's internal power differs from `^`
at some exponents. Libm `pow`, a correctly rounded pow, `EXP(t·LN v)` with the product in an x87 register, and
`1/EXP(-t·LN v)` all score lower over the case (T-2733 Outcome table).
Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (`CouponSchedule::price`, `duration_of`).

## Acceptance criteria
- [ ] More bit-exact cells in probes/bond-extreme-coupon-yield with no new known-failure lines in any baseline, or the
      candidates scored recorded in financial.md (time-boxed, decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD (T-1592), the cells T-2973 owns.

`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits.
