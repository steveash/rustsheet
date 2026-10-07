---
id: T-1007
title: "Probe subnormal flush edges: literal arguments, per-step flush, AVERAGEA/AVERAGEIF/SUBTOTAL(1)/AGGREGATE(1)"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's handling of subnormal literals and per-step flushing in PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE, and of the other average functions, is known and the engine matches it.

## Context
T-0987 flushes only the final result of PRODUCT, SUMPRODUCT, SUMSQ and AVERAGE. Unprobed: a subnormal literal argument (`PRODUCT(1E-308,1E+300)`, `AVERAGE(1E-308,1E+300)`), flushing each product or square rather than the total (`PRODUCT(1E-200,1E-110,1E+300)`, `SUMPRODUCT({1E-200,1},{1E-110,1E-300})`), and AVERAGEA, AVERAGEIF(S), SUBTOTAL(1/101), AGGREGATE(1). See docs/excel-semantics/numbers.md "Subnormal results become 0"; code: `arithmetic::flush_subnormal`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells for each form above with oracle values (use `(x)=0` companions or scaling back up); numbers.md updated; engine fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/subnormal-flush-edges` (16 forms x value/text/`=0` cells, excel-graph). Excel flushes subnormal literals, every running PRODUCT/SUMPRODUCT product, and the mean in AVERAGEA, AVERAGEIF(S), SUBTOTAL(1/101) and AGGREGATE(1). The engine now does the same (30/50 cells passed before; the remaining mismatch is the Graph-API scoring artifact for SUM of a kept subnormal, baselined, T-1027). Unit test added.
