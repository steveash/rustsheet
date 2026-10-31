---
id: T-2972
title: "DURATION and MDURATION bit-exact residue at a huge coupon rate and yield after T-2733 (43 cells of probes/bond-extreme-coupon-yield)"
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
DURATION and MDURATION give Excel's bits on probes/bond-extreme-coupon-yield, not only its 15 digits.

## Context
T-2733 found the sum order that brings every cell of the case within 15 significant digits (financial.md "PRICE,
DURATION and MDURATION sum order at a huge coupon rate and yield"). In the eval at T-2733 every PRICE cell of the case
is bit-exact (240 of 240), and the three gen-m9 cells T-2733 owned pass (no PRICE lead remains here). The 47
cells left (eval/baseline/probes.txt) are all DURATION (Calc column B) or MDURATION (column C), numeric and within
2 ulp. Four of them, Calc!B7, B43, C7 and C43, are T-2973's (T-2733 moved them by 1 or 2 ulp). This task owns the
other 43:
- DURATION, 18: Calc!B5, B12, B16, B18, B24, B27, B29, B32, B35, B67, B78, B85, B89, B91, B100, B184, B185, B192.
- MDURATION, 25: Calc!C5, C12, C16, C18, C24, C27, C29, C32, C35, C67, C78, C85, C89, C91, C100, C110, C118, C121,
  C123, C134, C138, C144, C184, C185, C192.
For example Calc!B192 `DURATION(43830,45293,1E+4,1E+3,4,1)`: Excel 0.006434782608695654 vs ours 0.006434782608695652
(2 ulp); Calc!C85 `MDURATION(38136,44737,1E+4,1,4,0)`: 0.8577765478446879 vs 0.8577765478446882 (2 ulp).
The redemption and coupon discounts already match Excel (every PRICE cell is bit-exact with the same `k + DSC/E`
exponents), so the lead is the order of the Macaulay sum, its division by the price, and MDURATION's division by
`1 + yld/f` (T-2732's mpmath comparison in financial.md "PRICE and ODDFPRICE discount with Excel's `^`": on the short
stubs Excel's DURATION and MDURATION are within 3.1 ulp of the exact value).
Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (`duration_of`).

## Acceptance criteria
- [ ] More bit-exact cells among the 43 above with no new known-failure lines in any baseline, or the candidates
      scored recorded in financial.md (time-boxed, decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PRICE (bit-exact on the case), YIELD (T-1592), the cells T-2973 owns (Calc!B7, B43, C7, C43, gen-m9
  5ded7594e597e31e Calc!A123).

`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits.
