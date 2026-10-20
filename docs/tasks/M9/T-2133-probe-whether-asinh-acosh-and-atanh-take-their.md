---
id: T-2133
title: "Probe whether ASINH, ACOSH and ATANH take their ln from FYL2X like LN"
milestone: M9
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether Excel's ASINH, ACOSH and ATANH take their logarithm from the x87 FYL2X (as LN does, T-2123) or from
something else, and match it.

## Context
T-2123 found LN = RN53(RN64(RN64(ln 2)·log2 x)) (numbers.md "LN is FLDLN2 then FYL2X"; `x87_log10.rs` `excel_ln`). ASINH,
ACOSH and ATANH (`functions/math_and_trigonometry/mathematical.rs`) still use the platform `ln`; found in T-2123's
Excel-fidelity review. Probe (excel-probe skill) a few hundred x per function where `f64::ln` and `excel_ln` lead to
different results, then switch only if the oracle says so.

## Acceptance criteria
- [ ] A probes case with excel-graph values for ASINH, ACOSH and ATANH at points where the two ln models differ.
- [ ] The engine follows the evidence; numbers.md records the rule.
- [ ] `make check` green; reviewers approve.

**ATANH is done (T-0917):** its log branch is `0.5 * excel_ln` of the twice-rounded quotient, fitted on 62 cells
(numbers.md "ATANH and ACOTH"); only ASINH and ACOSH remain here.

## Out of scope
- LN itself (T-2123) and the gamma functions (T-2128, T-2129).
- ATANH and ACOTH (T-0917).

## Outcome
Probe `probes-numeric/t2133-asinh-acosh-ln` (240 cells chosen where libm `ln` and `excel_ln` differ; excel-graph 2026-10-20): libm 2/240, `excel_ln` 238/240. ASINH and ACOSH now use `excel_ln`; the two one-ulp ACOSH misses (x = 20.82932287508296, 180.0774330018289) are baselined and recorded in numbers.md. ATANH was done by T-0917.
