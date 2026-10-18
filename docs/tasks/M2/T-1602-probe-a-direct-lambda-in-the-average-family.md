---
id: T-1602
title: "Probe a direct LAMBDA in the AVERAGE family and its error precedence"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for a direct LAMBDA in GEOMEAN, HARMEAN, VAR, STDEV, SKEW, KURT, DEVSQ, AVEDEV and SUMSQ, its rank against other errors, and a function name passed directly.

## Context
T-1572 gave the shared CastScalars rule #VALUE! for a direct LAMBDA (proved only for AVERAGE and MEDIAN). Untested: SUM(LAMBDA(x,x),#N/A), SUM(A5,LAMBDA(x,x)), SUM(_xleta.SUM). Use the excel-probe skill.

## Acceptance criteria
- [ ] Probe cases with oracle values cover these; syntax.md and SPEC-0009 updated and ours fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/aggregate-direct-lambda-average-family` (31 cells, excel-graph): the whole AVERAGE family gives #VALUE! for a direct LAMBDA, and that error ranks with the referenced errors (a later direct error beats it; the leftmost of it and a referenced error wins), also for DEVSQ and SUMSQ. A direct `_xleta.SUM` is #VALUE! for SUM and AVERAGE, skipped by COUNT. Ours returned the LAMBDA error at once; fixed with `AggregateWalk::lambda_error`. syntax.md and SPEC-0009 updated. Eval: new probe passes 31/31; baseline registers the new case.
