---
id: T-3032
title: "IMCOT last digit: three T-2787 probe cells miss"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMCOT prints Excel's text for the three probes/t2787-imcot-denominator-grouping cells it misses by one unit of the 15th digit.

## Context
- A2 `IMCOT("1.0791+0.6495i")` (expected ...187i, ours ...188i), A9 `IMCOT("2.2478-0.4516i")` (real ...611 vs ...612) and A17 `IMCOT("-2.9144+0.0359i")` (imaginary ...302 vs ...303).
- The T-2537 denominator grouping is confirmed (21 of 24 cells; the other grouping fits 2), so the misses are in the numerators or the division. Kernel: `imcot` in functions/engineering/complex/trig.rs; engineering.md IMCOT entry.

## Acceptance criteria
- [ ] three cells pass and eval/baseline shrinks accordingly, without regressing the 21 others or the 63 sweep cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
