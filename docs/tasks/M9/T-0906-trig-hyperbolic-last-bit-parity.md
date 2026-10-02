---
id: T-0906
title: "Trig/hyperbolic functions: last-bit parity with Excel (SIN/COS/TAN, SINH/TANH, COT/COTH/CSCH, ASINH/ACOTH...)"
milestone: M9
lane: functions
status: todo
priority: P1
size: M
depends_on: []
---

## Goal
Eliminate the ~136 failing cells in the TRIGONOMETRIC workbook caused by differing last-bit results
(plus argument-reduction mismatches for large arguments) for the basic trig/hyperbolic functions.
SIN/COS/TAN/SINH/TANH/EXP/LOG have very high real-world frequency, so this is the most valuable
numeric group.

## Context
All in `ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx` (136 baseline lines). Per function:
TANH 19, COTH 14, COT 17, ASINH 12, CSCH 9, SINH 8, ACOTH 14, TAN 8, SIN 6, COS 6, SEC 5, CSC 5,
ACOS 4, ATANH 4, ACOSH 3, COSH 1, SECH 1. Related cells in other files: EXP_SIGN.xlsx (1), LOG_LOG10_LN.xlsx (4), ATAN2_POWER.xlsx (4).
Examples: `TANH(A3)` expected 0.999909204262595, got 0.9999092042625951 (Excel appears to
round/truncate at 15 significant digits for some results, or computes derived functions via
different identities, e.g. COTH = 1/TANH vs (e^2x+1)/(e^2x-1)); SIN/COS/TAN NumericMismatch cells are
likely large-argument reduction differences. Group findings by hypothesis: (a) final 15-digit
rounding, (b) formula identity used for derived functions, (c) argument reduction. Test each against
all cells; implement the one that fits. Do not regress the cells that currently pass.
Run `make eval-show CASE=ironcalc-calc/MATH_AND_TRIGONOMETRY/TRIGONOMETRIC.xlsx`.
Code: `crates/rustsheet-engine/src/functions/math_and_trigonometry/`.

## Acceptance criteria
- [ ] At least the 100 baseline lines for the hyperbolic and reciprocal-trig functions (TANH, COTH, COT, CSCH,
      SINH, ASINH, ACOTH, SEC, CSC, SECH, COSH, ACOSH, ATANH, ACOS) disappear, no new failures; remaining
      SIN/COS/TAN lines may stay if split to a follow-up task.
- [ ] Hypothesis and evidence recorded in docs/excel-semantics/; `make check` green.

## Out of scope
Complex-number IM* functions (T-0907); statistical distributions (T-0903).
