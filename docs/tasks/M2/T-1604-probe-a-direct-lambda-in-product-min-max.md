---
id: T-1604
title: "Probe a direct LAMBDA in PRODUCT, MIN, MAX and before an array error"
milestone: M2
lane: research
status: done
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

## Outcome
Probe probes/aggregate-direct-lambda-product-min-max (36 cells, excel-graph) shows PRODUCT, MIN and MAX rank a direct LAMBDA's #VALUE! like SUM. Ours returned it at once (6 cells wrong in PRODUCT/MIN/MAX plus 2 array-error cells in MIN/MAX); `fold_min_max` and `fn_product` now hold it. Eval: probe 36/36, 0 regressions.
