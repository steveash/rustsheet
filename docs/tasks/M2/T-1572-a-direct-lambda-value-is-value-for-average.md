---
id: T-1572
title: "A direct LAMBDA value is #VALUE! for AVERAGE, MEDIAN, SUM, PRODUCT, MIN and MAX (COUNT skips it)"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A direct LAMBDA argument gives #VALUE! (ERROR.TYPE 3) in SUM, PRODUCT, MIN, MAX, AVERAGE and MEDIAN; COUNT keeps skipping it.

## Context
probes/aggregate-picked-direct-a-variants D7-D13 (ours: #CALC! (102) for SUM/PRODUCT/MIN/MAX, and AVERAGE/MEDIAN skip it, giving 2). syntax.md has the rule; SPEC-0009 rows SUM/MIN/MAX/PRODUCT and COUNT.

## Acceptance criteria
- [x] The probe cells named in Context pass (`rustsheet-eval show`), no new known failures; the SPEC-0009 row is updated from DIVERGES to matched.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cases the probe did not cover.

## Outcome
A direct LAMBDA is `#VALUE!` in SUM, PRODUCT, MIN, MAX (`cast_aggregate_arg_to_number`) and AVERAGE, MEDIAN (the `CastScalars` direct rule); COUNT still skips it. probes/aggregate-picked-direct-a-variants D7-D12 pass; probes known failures 5581 -> 5579 on top of main (no regressions). The rest of the AVERAGE family now also gives `#VALUE!` (shared rule, unprobed, noted in syntax.md).
