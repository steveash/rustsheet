---
id: T-1057
title: "Probe SUMSQ per-square subnormal flush and SUMPRODUCT with three arrays"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's handling of a subnormal square inside SUMSQ, and of a subnormal product across three or more SUMPRODUCT arrays, is known and the engine matches it.

## Context
T-1007 verified per-step flushing for PRODUCT and for two-array SUMPRODUCT (probes/subnormal-flush-edges). SUMSQ still flushes only the final total (`SUMSQ(1E-155,1E-150)` tells per-square from total flushing), and the three-array SUMPRODUCT fold flushes between factors without an oracle cell. See docs/excel-semantics/numbers.md "Subnormal results become 0"; code: `fn_sumsq`, `fn_sumproduct`.

## Acceptance criteria
- [ ] Probe cells for SUMSQ per-square flushing and a three-array SUMPRODUCT with oracle values; numbers.md updated; engine fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other subnormal edges already probed by T-1007.

## Outcome
Probe `probes/subnormal-sumsq-sumproduct-edges` (7 forms x value/text/`=0` cells, excel-graph): Excel flushes each SUMSQ square and each SUMPRODUCT factor left to right. The three-array SUMPRODUCT already matched; SUMSQ flushed only the total (15/21 cells before), so it now flushes per square. Unit test added. Eval: new case 21/21.
