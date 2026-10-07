---
id: T-0268
title: "ROUNDUP with a wrapped num_digits is one ulp above the decimal result (3 probes known failures)"
milestone: M2
lane: functions-math
status: todo
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
`effective_digits` / `round_decimal` in functions/math_and_trigonometry/decimal_round.rs.

## Acceptance criteria
- [ ] The three cells pass, with the rule recorded in numbers.md (VERIFIED by new probe cells).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit differences of ROUND with ordinary num_digits (T-0106).
