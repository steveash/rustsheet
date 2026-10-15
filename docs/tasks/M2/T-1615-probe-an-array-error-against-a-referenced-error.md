---
id: T-1615
title: "Probe an array error against a referenced error in MIN, MAX, AVERAGE and MEDIAN"
milestone: M2
lane: research
status: done
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
- [x] A probe covers `fn(A6,{1,#N/A})`, `fn({1,#N/A},A6)`, `fn(A7:A8,{1,#N/A})` and `fn(LAMBDA(x,x),{1,#N/A})` for MIN, MAX, AVERAGE, MEDIAN with oracle values; syntax.md updated; if array errors rank as referenced, drop `lambda_held` in `fold_min_max` and hold the array error.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/aggregate-array-error-vs-ref-error (40 cells, excel-graph, 2026-10-15): in SUM, MIN, MAX, AVERAGE and MEDIAN an array error ranks with the referenced errors by position. Ours returned it at once after a referenced error in MIN, MAX, AVERAGE and MEDIAN after a referenced error or a direct LAMBDA (10 of 25 cells); a direct error beats an array error either way (D26-D40, ours already matched); fixed by removing `ArrayErrorRank` and `lambda_held` so the walker holds array errors for every `Ranked` policy. Eval: probes +1 case, 40/40 pass, 0 regressions.

Follow-up: T-1629 probes the rest of the AVERAGE family, which shares the walker.
