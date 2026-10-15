---
id: T-1602
title: "Probe a direct LAMBDA in the AVERAGE family and its error precedence"
milestone: M2
lane: research
status: todo
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
