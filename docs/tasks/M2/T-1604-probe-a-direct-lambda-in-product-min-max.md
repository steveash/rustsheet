---
id: T-1604
title: "Probe a direct LAMBDA in PRODUCT, MIN, MAX and before an array error"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how a direct LAMBDA's #VALUE! ranks in PRODUCT, MIN and MAX against direct and referenced errors, and before an array error.

## Context
T-1602 (probes/aggregate-direct-lambda-average-family) showed the LAMBDA's #VALUE! ranks with referenced errors in SUM, AVERAGE, DEVSQ and SUMSQ; `AggregateWalk::lambda_error` implements it in the shared walker. PRODUCT, MIN and MAX may not use that path, and `SUM(LAMBDA(x,x),{1,#N/A})` (array error after a LAMBDA) is unprobed. Use the excel-probe skill; use an A6 error other than #VALUE! so the winner is visible.

## Acceptance criteria
- [ ] A probe case with oracle values covers PRODUCT, MIN, MAX and SUM with a LAMBDA before/after #N/A, a referenced error and an array error; syntax.md and SPEC-0009 updated and ours fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
