---
id: T-0268
title: "ROUNDUP with a wrapped num_digits is one ulp above the decimal result (3 probes known failures)"
milestone: M2
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ROUNDUP with a `num_digits` that Excel wraps (docs/excel-semantics/numbers.md "ROUND family: huge
num_digits wrap") returns Excel's exact double, including its last ulp.

## Context
T-0218 found that `ROUNDUP(123.456,4294901762)`, `ROUNDUP(123.456,2147549186)` and
`ROUNDUP(123.456,2147483650)` (digits 2 after wrapping) are 123.46000000000001 in Excel, one ulp
above `ROUNDUP(123.456,2)` = 123.46, while ROUNDDOWN at the same digits matches the decimal result.
That looks like a binary `x * 10^-k` path (12346 * 0.01 rounds to 123.46000000000001) used only
when the digit count was wrapped. Probe more wrapped digit counts and values with the
`excel-probe` skill to find which path and when, before implementing. Cells:
probes/round-huge-digits-wrap Sheet1!A16, A35, A38 (NumericUlp). Code:
`effective_digits` / `round_decimal` in decimal_round.rs.

## Acceptance criteria
- [x] The three cells pass, with the rule recorded in numbers.md (VERIFIED by new probe cells).
- [x] `make check` green; reviewers approve.

## Outcome
The wrap was a red herring: Excel's ROUNDUP always rounds down in decimal, converts that to a double
and adds `10^-num_digits` as a double, so plain `ROUNDUP(123.456,2)` is 123.46000000000001 too.
New probe probes/roundup-wrapped-scaling (480 cells, excel-graph) fits the rule on 80/80 values with
wrapped and plain digits alike; rejected models are in numbers.md "ROUNDUP adds the unit in
binary". `round_decimal` implements it for ROUNDUP only. Eval: probes fixed 3 (round-huge-digits-wrap
A16, A35, A38) + new case all passing; ironcalc-calc fixed 1 (ROUND.xlsx D6); 0 regressions.

## Out of scope
- Last-digit differences of ROUND with ordinary num_digits (T-0106).
