---
id: T-1615
title: "Probe an array error against a referenced error in MIN, MAX, AVERAGE and MEDIAN"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how an error inside an array ranks against a referenced error in MIN, MAX, AVERAGE and MEDIAN, and ours fixed if it differs.

## Context
T-1604 (probes/aggregate-direct-lambda-product-min-max D15, D24) showed `MIN(LAMBDA(x,x),{1,#N/A})` is #VALUE!, so a MIN/MAX array error loses to an earlier LAMBDA error. `fold_min_max` special-cases that with `lambda_held`, while an array error still returns at once over a held referenced error (`MIN(A6,{1,#N/A})`), which is unprobed. SUM and PRODUCT rank array errors with referenced errors. AVERAGE and MEDIAN (`ArrayErrorRank::Direct`) probably differ from MIN/MAX for `AVERAGE(LAMBDA(x,x),{1,#N/A})`. Use the excel-probe skill; use A5 #N/A, A6 #VALUE!, A7 #DIV/0!.

## Acceptance criteria
- [ ] A probe covers `fn(A6,{1,#N/A})`, `fn({1,#N/A},A6)`, `fn(A7:A8,{1,#N/A})` and `fn(LAMBDA(x,x),{1,#N/A})` for MIN, MAX, AVERAGE, MEDIAN with oracle values; syntax.md updated; if array errors rank as referenced, drop `lambda_held` in `fold_min_max` and hold the array error.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
